use bevy::{
    asset::RenderAssetUsages,
    color::palettes::css::{AQUA, BLACK, WHITE},
    platform::collections::{HashMap, HashSet},
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
    window::PrimaryWindow,
};
use hexx::{algorithms::a_star, ColumnMeshBuilder, Hex, HexLayout};

/// 场景
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_camera, setup_grid))
            .add_systems(Update, handle_input);
    }
}

fn setup_camera(mut commands: Commands) {
    // 添加环境光
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.5,
        affects_lightmapped_meshes: false,
    });

    // 添加定向光（模拟太阳光）
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 60.0, 60.0).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Perspective(PerspectiveProjection {
            fov: 45.0f32.to_radians(),
            ..default()
        }),
    ));
}

#[derive(Debug, Resource)]
struct Map {
    layout: HexLayout,
    entities: HashMap<Hex, Entity>,
    blocked_coords: HashSet<Hex>,
    path_entities: HashSet<Entity>,
    blocked_material: Handle<StandardMaterial>,
    default_material: Handle<StandardMaterial>,
    path_material: Handle<StandardMaterial>,
}

fn setup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let layout = HexLayout {
        scale: HEX_SIZE,
        ..default()
    };
    // materials
    let default_material = materials.add(Color::Srgba(WHITE));
    let blocked_material = materials.add(Color::Srgba(BLACK));
    let path_material = materials.add(Color::Srgba(AQUA));
    // mesh
    let mesh = hexagonal_column(&layout);
    let mesh_handle = meshes.add(mesh);
    let mut blocked_coords = HashSet::new();
    let entities = Hex::ZERO
        .spiral_range(0..=MAP_RADIUS)
        .enumerate()
        .map(|(i, hex)| {
            let pos = layout.hex_to_world_pos(hex);
            let material = match hex {
                c if i != 0 && i % 5 == 0 => {
                    blocked_coords.insert(c);
                    blocked_material.clone()
                }
                _ => default_material.clone(),
            };
            let height = if i != 0 && i % 5 == 0 {
                -COLUMN_HEIGHT + 1.0
            } else {
                -COLUMN_HEIGHT
            };
            let id = commands
                .spawn((
                    Mesh3d(mesh_handle.clone()),
                    MeshMaterial3d(material.clone_weak()),
                    Transform::from_xyz(pos.x, height, pos.y),
                ))
                .id();
            (hex, id)
        })
        .collect();
    commands.insert_resource(Map {
        layout,
        entities,
        blocked_coords,
        path_entities: Default::default(),
        default_material,
        blocked_material,
        path_material,
    });
}

/// Input interaction
fn handle_input(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut current: Local<Hex>,
    mut grid: ResMut<Map>,
) -> Result {
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
    let Some(entity) = grid.entities.get(&hex_pos).copied() else {
        return Ok(());
    };
    if buttons.just_pressed(MouseButton::Left) {
        if grid.blocked_coords.contains(&hex_pos) {
            grid.blocked_coords.remove(&hex_pos);
            commands
                .entity(entity)
                .insert(MeshMaterial3d(grid.default_material.clone_weak()));
        } else {
            grid.blocked_coords.insert(hex_pos);
            grid.path_entities.remove(&entity);
            commands
                .entity(entity)
                .insert(MeshMaterial3d(grid.blocked_material.clone_weak()));
        }
        return Ok(());
    }
    if hex_pos == *current {
        return Ok(());
    }
    *current = hex_pos;
    let path_to_clear: Vec<_> = grid.path_entities.drain().collect();
    for entity in path_to_clear {
        commands
            .entity(entity)
            .insert(MeshMaterial3d(grid.default_material.clone_weak()));
    }

    // let check_pos = Hex::new(hex_pos.x, hex_pos.z);
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
    for entity in &entities {
        commands
            .entity(*entity)
            .insert(MeshMaterial3d(grid.path_material.clone_weak()));
    }
    grid.path_entities = entities;

    Ok(())
}

/// World size of the hexagons (outer radius)
const HEX_SIZE: Vec2 = Vec2::splat(1.0);
/// World space height of hex columns
const COLUMN_HEIGHT: f32 = 10.0;
/// Map radius
const MAP_RADIUS: u32 = 20;

/// Compute a bevy mesh from the layout
fn hexagonal_column(hex_layout: &HexLayout) -> Mesh {
    let mesh_info = ColumnMeshBuilder::new(hex_layout, COLUMN_HEIGHT)
        .without_bottom_face()
        .center_aligned()
        .build();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs)
    .with_inserted_indices(Indices::U16(mesh_info.indices))
}
