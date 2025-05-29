use crate::character::MapHex;
use bevy::{
    asset::RenderAssetUsages,
    color::palettes::css::{AQUA, BLACK, WHITE},
    platform::collections::{HashMap, HashSet},
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use hexx::{ColumnMeshBuilder, Hex, HexLayout};
use std::{collections::VecDeque, f32::consts::PI};

/// 场景
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_camera, setup_grid));
        // .add_systems(Update, handle_input);
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

    // 创建类似Diablo的45度等轴测视角
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(20.0, 20.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Perspective(PerspectiveProjection {
            fov: 45.0f32.to_radians(),
            ..default()
        }),
    ));
}

#[derive(Debug, Resource)]
pub(crate) struct Map {
    pub(crate) layout: HexLayout,
    pub(crate) entities: HashMap<Hex, Entity>,
    pub(crate) blocked_coords: HashSet<Hex>,
    pub(crate) path_entities: VecDeque<Entity>,
    blocked_material: Handle<StandardMaterial>,
    pub(crate) default_material: Handle<StandardMaterial>,
    path_material: Handle<StandardMaterial>,
}

fn setup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
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
            let blocked = i != 0 && i % 5 == 0;
            let material = match hex {
                c if blocked => {
                    blocked_coords.insert(c);
                    blocked_material.clone()
                }
                _ => default_material.clone(),
            };
            let height = if blocked {
                -COLUMN_HEIGHT
            } else {
                -COLUMN_HEIGHT
            };
            let mesh_path = if blocked {
                "starter_kit/stone-hill.glb"
            } else {
                "starter_kit/sand.glb"
            };
            let id = commands
                .spawn((
                    Mesh3d(mesh_handle.clone()),
                    MeshMaterial3d(material.clone_weak()),
                    Transform::from_xyz(pos.x, height, pos.y),
                    MapHex(hex),
                ))
                .with_children(|parent| {
                    // parent.spawn((RigidBody::Static, Collider::cuboid(1.0, 1.0, 1.0)));
                    parent.spawn((
                        SceneRoot(
                            asset_server.load(GltfAssetLabel::Scene(0).from_asset(mesh_path)),
                        ),
                        Transform::from_xyz(0.0, -height, 0.0)
                            .with_scale(Vec3::splat(1.8))
                            .with_rotation(Quat::from_rotation_y(PI / 2.0)),
                    ));
                })
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

/// World size of the hexagons (outer radius)
const HEX_SIZE: Vec2 = Vec2::splat(1.0);
/// World space height of hex columns
const COLUMN_HEIGHT: f32 = 1.0;
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
