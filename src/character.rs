use avian3d::prelude::{Collider, RigidBody};
use bevy::{prelude::*, window::PrimaryWindow};
use hexx::{algorithms::a_star, Hex};
use std::collections::VecDeque;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (click_to_move, move_character));
    }
}

#[derive(Component)]
pub struct Character {
    pub move_speed: f32,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("Character"),
        SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("starter_kit/character.glb")),
        ),
        Transform::from_xyz(0.0, 0.5, 0.0),
        Character {
            // 初始化速度
            move_speed: 5.0,
        },
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0, 1.0),
    ));
}

#[derive(Component, Deref)]
pub struct MapHex(pub Hex);

fn click_to_move(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), Without<Character>>,
    mut current: Local<Hex>,
    mut grid: ResMut<crate::scene::Map>,
    q_character: Query<(Entity, &Transform, &Character), Without<Camera>>,
) -> Result {
    if buttons.just_pressed(MouseButton::Left) {
        let window = windows.single()?;

        let (camera, cam_transform) = cameras.single()?;
        let Some(ray) = window
            .cursor_position()
            .and_then(|p| camera.viewport_to_world(cam_transform, p).ok())
        else {
            return Ok(());
        };
        let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Dir3::Y)) else {
            return Ok(());
        };
        let point = ray.origin + ray.direction * distance;
        let hex_pos = grid.layout.world_pos_to_hex(point.xz());
        if hex_pos == *current {
            return Ok(());
        }
        *current = hex_pos;

        // 获取角色实体
        if let Ok((_character_entity, transform, _)) = q_character.single() {
            // 使用新的move_character_to_hex函数移动角色
            if let Err(err) = move_character_to_hex(hex_pos, transform, &mut grid) {
                error!("移动角色失败: {}", err);
            } else {
                info!("角色开始移动到 {:?}", hex_pos);
            }
        } else {
            error!("未找到角色实体");
        }
    }

    Ok(())
}

// 计算平滑移动的速度
fn calculate_velocity(current_pos: Vec3, target_pos: Vec3, speed: f32) -> Vec3 {
    let direction = target_pos - current_pos;
    let distance = direction.length();

    // 如果距离足够小，返回零向量
    if distance < 0.01 {
        return Vec3::ZERO;
    }

    // 计算当前速度
    direction.normalize() * speed
}

fn move_character(
    time: Res<Time>,
    mut q_character: Query<(&mut Transform, &Character)>,
    mut grid: ResMut<crate::scene::Map>,
    mut local_target: Local<Option<Entity>>,
    mut target_position: Local<Option<Vec3>>, // 只存储目标位置
    q_hex: Query<(&Transform, &MapHex), Without<Character>>,
) -> Result<()> {
    let (mut transform, character) = q_character.single_mut()?;

    // 如果有目标实体
    if let Some(target_entity) = *local_target {
        // 获取目标实体的变换
        let Ok((hex_transform, _)) = q_hex.get(target_entity) else {
            *local_target = None;
            *target_position = None;
            return Ok(());
        };

        // 创建目标位置，保持Y坐标不变
        let current_target_pos = Vec3::new(
            hex_transform.translation.x,
            transform.translation.y,
            hex_transform.translation.z,
        );

        // 如果这是新目标，初始化目标位置
        if target_position.is_none() {
            *target_position = Some(current_target_pos);
            info!("开始移动到新目标: {:?}", current_target_pos);
        }

        let target_pos = target_position.unwrap();
        let current_pos = transform.translation;

        // 计算当前位置到目标的距离
        let distance_vec = target_pos - current_pos;
        let distance = distance_vec.length();

        // 如果距离很小，认为已经到达目标
        if distance < 0.1 {
            info!("已到达目标位置");
            // 直接设置到目标位置
            transform.translation = target_pos;
            // 清除目标
            *local_target = None;
            *target_position = None;
            return Ok(());
        }

        // 计算移动速度，使用恒定速度
        let velocity = calculate_velocity(current_pos, target_pos, character.move_speed);

        // 计算新位置
        let new_position = current_pos + velocity * time.delta_secs();

        // 检查是否会超过目标
        let new_distance = (target_pos - new_position).length();

        // 如果新位置比当前位置离目标更远，或者新距离很小，直接到达目标
        if new_distance > distance || new_distance < 0.1 {
            transform.translation = target_pos;
            *local_target = None;
            *target_position = None;
            info!("到达目标位置");
        } else {
            // 更新角色位置
            transform.translation = new_position;
            info!(
                "移动中: 当前位置: {:?}, 目标位置: {:?}, 距离: {:.2}",
                new_position, target_pos, new_distance
            );
        }
    } else {
        // 如果没有当前目标，尝试获取新目标
        if let Some(first_entity) = grid.path_entities.pop_front() {
            *local_target = Some(first_entity);
        }
    }

    Ok(())
}

/// 从一个hex移动到另一个hex
///
/// # 参数
/// * `from_hex` - 起始hex坐标
/// * `to_hex` - 目标hex坐标
/// * `grid` - 地图资源
///
/// # 返回
/// * `Result<Vec<Entity>>` - 成功时返回路径上的实体列表，失败时返回错误
pub fn move_to_hex(
    from_hex: Hex,
    to_hex: Hex,
    grid: &mut crate::scene::Map,
) -> Result<Vec<Entity>> {
    // 如果起点和终点相同，直接返回空路径
    if from_hex == to_hex {
        return Ok(Vec::new());
    }

    // 使用A*算法计算路径
    let Some(path) = a_star(from_hex, to_hex, |_, h| {
        // 只有当hex存在且不被阻挡时才能通过
        (grid.entities.contains_key(&h) && !grid.blocked_coords.contains(&h)).then_some(1)
    }) else {
        info!("无法找到从 {:?} 到 {:?} 的路径", from_hex, to_hex);
        return Ok(vec![]);
    };

    info!("找到从 {:?} 到 {:?} 的路径: {:?}", from_hex, to_hex, path);

    // 将路径转换为实体列表
    let entities: Vec<_> = path
        .into_iter()
        .filter_map(|h| grid.entities.get(&h).copied())
        .collect();

    // 更新地图的路径实体
    grid.path_entities = VecDeque::from(entities.clone());

    Ok(entities)
}

/// 获取角色当前所在的hex坐标
pub fn get_character_hex(character_transform: &Transform, grid: &crate::scene::Map) -> Hex {
    // 从世界坐标转换为hex坐标
    grid.layout
        .world_pos_to_hex(character_transform.translation.xz())
}

/// 移动角色到指定的hex坐标
pub fn move_character_to_hex(
    target_hex: Hex,
    transform: &Transform,
    grid: &mut crate::scene::Map,
) -> Result<()> {
    // 获取当前hex坐标
    let current_hex = get_character_hex(transform, grid);
    info!(
        "角色当前位置: {:?}, 目标位置: {:?}",
        current_hex, target_hex
    );

    // 计算路径
    let path = move_to_hex(current_hex, target_hex, grid)?;

    // 如果路径为空，则无需移动
    if path.is_empty() {
        return Ok(());
    }

    // 更新地图的路径实体
    grid.path_entities = VecDeque::from(path);

    Ok(())
}
