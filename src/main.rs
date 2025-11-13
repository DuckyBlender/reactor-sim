use bevy::prelude::*;
use bevy::window::PrimaryWindow;

const PARALLAX_FACTOR: f32 = 0.05;
const CAMERA_DISTANCE: f32 = 15.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, camera_parallax)
        .run();
}

#[derive(Component)]
struct ParallaxCamera;

#[derive(Resource)]
struct MousePosition {
    normalized: Vec2,
}

impl Default for MousePosition {
    fn default() -> Self {
        Self {
            normalized: Vec2::ZERO,
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.init_resource::<MousePosition>();

    // Camera positioned to look at the cubes
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, CAMERA_DISTANCE)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ParallaxCamera,
    ));

    // Directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 200.0,
        affects_lightmapped_meshes: false,
    });

    // Create cube mesh
    let cube_mesh = meshes.add(Cuboid::new(1.5, 1.5, 1.5));
    
    // Create 9 cubes in a 3x3 grid
    let spacing = 2.5;
    let colors = [
        Color::srgb(0.8, 0.2, 0.2), // Red
        Color::srgb(0.2, 0.8, 0.2), // Green
        Color::srgb(0.2, 0.2, 0.8), // Blue
        Color::srgb(0.8, 0.8, 0.2), // Yellow
        Color::srgb(0.8, 0.2, 0.8), // Magenta
        Color::srgb(0.2, 0.8, 0.8), // Cyan
        Color::srgb(0.9, 0.5, 0.2), // Orange
        Color::srgb(0.5, 0.2, 0.9), // Purple
        Color::srgb(0.9, 0.9, 0.9), // White
    ];

    for i in 0..3 {
        for j in 0..3 {
            let x = (i as f32 - 1.0) * spacing;
            let y = (j as f32 - 1.0) * spacing;
            let color_index = i * 3 + j;
            
            commands.spawn((
                Mesh3d(cube_mesh.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: colors[color_index],
                    metallic: 0.3,
                    perceptual_roughness: 0.5,
                    ..default()
                })),
                Transform::from_xyz(x, y, 0.0),
            ));
        }
    }
}

fn camera_parallax(
    mut mouse_position: ResMut<MousePosition>,
    mut camera_query: Query<&mut Transform, With<ParallaxCamera>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };
    
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    
    // Normalize cursor position to range [-1, 1]
    let normalized_x = (cursor_position.x / window.width()) * 2.0 - 1.0;
    let normalized_y = -((cursor_position.y / window.height()) * 2.0 - 1.0);
    
    mouse_position.normalized = Vec2::new(normalized_x, normalized_y);
    
    // Update camera position with parallax effect
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        let offset_x = normalized_x * PARALLAX_FACTOR * CAMERA_DISTANCE;
        let offset_y = normalized_y * PARALLAX_FACTOR * CAMERA_DISTANCE;
        
        camera_transform.translation = Vec3::new(offset_x, offset_y, CAMERA_DISTANCE);
        camera_transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}
