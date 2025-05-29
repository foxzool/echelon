use bevy::{
    platform::collections::HashSet, prelude::*, render::primitives::Aabb, window::PrimaryWindow,
};
use hexx::{algorithms::a_star, Hex};

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, click_to_move);
    }
}

#[derive(Component)]
pub struct Character {
    pub move_speed: f32,
    pub rotation_speed: f32,
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("Character"),
        SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("starter_kit/character.glb")),
        ),
        Transform::from_xyz(0.0, 0.5, 0.0),
        Character {
            // 初始化速度
            move_speed: 5.0,
            rotation_speed: f32::to_radians(90.0), // 每秒旋转90度
        },
        // RigidBody::Dynamic,
        // Collider::cuboid(0.5, 1.5, 0.5),
    ));
}

fn click_to_move(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut current: Local<Hex>,
    mut grid: ResMut<crate::scene::Map>,
    mut query: Query<(&mut Transform, &Character)>,
) -> Result {
    if buttons.just_pressed(MouseButton::Left) {
        let window = windows.single()?;
        let (mut character_transform, character) = query.single_mut()?;
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
        let Some(entity) = grid.entities.get(&hex_pos).copied() else {
            return Ok(());
        };
        if hex_pos == *current {
            return Ok(());
        }
        *current = hex_pos;

        let Some(path) = a_star(Hex::ZERO, hex_pos, |_, h| {
            (grid.entities.contains_key(&h) && !grid.blocked_coords.contains(&h)).then_some(1)
        }) else {
            info!("No path found {:?}", hex_pos);
            return Ok(());
        };
        let entities: HashSet<_> = path
            .into_iter()
            .inspect(|h| {
                if grid.blocked_coords.contains(h) {
                    error!("A star picked a blocked coord: {h:?}");
                }
            })
            .filter_map(|h| grid.entities.get(&h).copied())
            .collect();
        println!("{:?}", entities.iter().last());
        grid.path_entities = entities;
    }

    Ok(())
}
