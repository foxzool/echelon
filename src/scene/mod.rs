use avian3d::prelude::{Collider, RigidBody};
use bevy::{color::palettes::basic::SILVER, prelude::*};

/// 场景
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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

    // 创建45度视角的相机
    let camera_position = Vec3::new(5.0, 5.0, 10.0);
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(camera_position.x, camera_position.y, camera_position.z)
            .looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Perspective(PerspectiveProjection {
            fov: 45.0f32.to_radians(),
            ..default()
        }),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::from(SILVER))),
        RigidBody::Static,
        Collider::cuboid(50.0, 0.01, 50.0),
    ));
}
