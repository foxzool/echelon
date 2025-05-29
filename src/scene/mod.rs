use bevy::{
    asset::RenderAssetUsages,
    color::palettes::css::{WHITE, YELLOW},
    platform::collections::HashMap,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
    window::PrimaryWindow,
};
use hexx::{shapes, ColumnMeshBuilder, Hex, HexLayout};

/// 场景
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_camera, setup_grid))
            .add_systems(Update, higlight_hovered);
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
    highlighted_material: Handle<StandardMaterial>,
    default_material: Handle<StandardMaterial>,
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
    let highlighted_material = materials.add(Color::Srgba(YELLOW));
    // mesh
    let mesh = hexagonal_column(&layout);
    let mesh_handle = meshes.add(mesh);

    let entities = shapes::hexagon(Hex::ZERO, MAP_RADIUS)
        .map(|hex| {
            let pos = layout.hex_to_world_pos(hex);
            let id = commands
                .spawn((
                    Mesh3d(mesh_handle.clone()),
                    MeshMaterial3d(default_material.clone_weak()),
                    Transform::from_xyz(pos.x, -COLUMN_HEIGHT, pos.y),
                ))
                .id();
            (hex, id)
        })
        .collect();
    commands.insert_resource(Map {
        layout,
        entities,
        highlighted_material,
        default_material,
    });
}

fn higlight_hovered(
    mut commands: Commands,
    map: Res<Map>,
    mut highlighted: Local<Hex>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window, With<PrimaryWindow>>,
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
    let coord = map.layout.world_pos_to_hex(point.xz());
    if coord != *highlighted {
        let Some(entity) = map.entities.get(&coord).copied() else {
            return Ok(());
        };
        commands
            .entity(entity)
            .insert(MeshMaterial3d(map.highlighted_material.clone_weak()));
        commands
            .entity(map.entities[&*highlighted])
            .insert(MeshMaterial3d(map.default_material.clone_weak()));
        *highlighted = coord;
    }
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
