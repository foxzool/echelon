use bevy::{prelude::*, render::primitives::Aabb};

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, character_movement_system); // 添加新的系统

        if cfg!(feature = "debug") {
            app.add_systems(Update, draw_axes);
        }
    }
}

#[derive(Component)]
pub struct Character {
    pub move_speed: f32,
    pub rotation_speed: f32,
}

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
        Character {
            // 初始化速度
            move_speed: 5.0,
            rotation_speed: f32::to_radians(90.0), // 每秒旋转90度
        },
    ));
}

fn character_movement_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &Character)>,
) {
    for (mut transform, character) in query.iter_mut() {
        let mut direction = Vec3::ZERO;
        let mut rotation = 0.0;

        if keyboard_input.pressed(KeyCode::KeyW) {
            direction += *transform.forward();
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction += *transform.back();
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction += *transform.left();
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction += *transform.right();
        }

        if keyboard_input.pressed(KeyCode::KeyQ) {
            rotation += character.rotation_speed * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyE) {
            rotation -= character.rotation_speed * time.delta_secs();
        }

        if direction.length_squared() > 0.0 {
            direction = direction.normalize();
            transform.translation += direction * character.move_speed * time.delta_secs();
        }

        if rotation != 0.0 {
            transform.rotate_y(rotation);
        }
    }
}

fn draw_axes(mut gizmos: Gizmos, query: Query<(&Transform, &Aabb), With<Character>>) {
    for (&transform, &aabb) in &query {
        let length = aabb.half_extents.length();
        gizmos.axes(transform, length);
    }
}
