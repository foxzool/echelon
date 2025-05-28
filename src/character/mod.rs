use avian3d::prelude::{Collider, RigidBody};
use bevy::{prelude::*, render::primitives::Aabb};

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);

        if cfg!(feature = "debug") {
            app.add_systems(Update, draw_axes);
        }
    }
}

#[derive(Component)]
pub struct Character;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let shape = meshes.add(Capsule3d::default());
    commands.spawn((
        Mesh3d(shape),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(0.0, 5.0, 0.0),
        Character,
        RigidBody::Dynamic,
        Collider::capsule(0.5, 1.0)
    ));
}

fn draw_axes(mut gizmos: Gizmos, query: Query<(&Transform, &Aabb), With<Character>>) {
    for (&transform, &aabb) in &query {
        let length = aabb.half_extents.length();
        gizmos.axes(transform, length);
    }
}
