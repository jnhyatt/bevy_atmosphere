//! Based off the Bevy "Split Screen" example
//! Used to demonstrate how multiple skyboxes could be made for a local multiplayer game

use bevy::{
    prelude::*,
    render::{camera::Viewport, view::RenderLayers},
    window::WindowResized,
};
use bevy_atmosphere::prelude::*;
use bevy_spectator::{Spectator, SpectatorPlugin, SpectatorSettings};

fn main() {
    println!("Demonstrates using `AtmosphereCamera.render_layers` to have multiple skyboxes in the scene at once\n\t- E: Switch camera");
    App::new()
        .insert_resource(AtmosphereModel::new(Nishita {
            rayleigh_coefficient: Vec3::new(22.4e-6, 5.5e-6, 13.0e-6), // Change rayleigh coefficient to change color
            ..default()
        }))
        .insert_resource(SpectatorSettings {
            base_speed: 0.5,
            alt_speed: 1.0,
            ..default()
        })
        .add_plugins((DefaultPlugins, AtmospherePlugin, SpectatorPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, set_camera_viewports)
        .add_systems(Update, switch_camera)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut settings: ResMut<SpectatorSettings>,
) {
    // Plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            1.0,
            -std::f32::consts::FRAC_PI_4,
        )),
    ));

    // Spawn left screen camera and make it the default spectator
    let left = commands
        .spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 25.0, -100.0).looking_at(Vec3::ZERO, Vec3::Y),
            Msaa::Sample4,
            RenderLayers::from_layers(&[0, 1]), // To prevent each player from seeing the other skybox, we put each one on a separate render layer (you could also use this render layer for other player specific effects)
            AtmosphereCamera {
                render_layers: Some(RenderLayers::layer(1)),
            },
            LeftCamera,
            Spectator,
        ))
        .id();

    settings.active_spectator = Some(left);

    // Spawn right screen camera
    commands.spawn((
        Camera {
            // Renders the right camera after the left camera, which has a default priority of 0
            order: 1,
            // Don't clear on the second camera because the first camera already cleared the window
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Camera3d::default(),
        Transform::from_xyz(100.0, 50.0, 150.0).looking_at(Vec3::ZERO, Vec3::Y),
        Msaa::Sample4,
        RenderLayers::from_layers(&[0, 2]), // To prevent each player from seeing the other skybox, we put each one on a separate render layer (you could also use this render layer for other player specific effects)
        AtmosphereCamera {
            render_layers: Some(RenderLayers::layer(2)),
        },
        RightCamera,
        Spectator,
    ));
}

#[derive(Component)]
struct LeftCamera;

#[derive(Component)]
struct RightCamera;

fn set_camera_viewports(
    windows: Query<&Window>,
    mut resize_events: EventReader<WindowResized>,
    mut left_camera: Query<&mut Camera, (With<LeftCamera>, Without<RightCamera>)>,
    mut right_camera: Query<&mut Camera, With<RightCamera>>,
) {
    // We need to dynamically resize the camera's viewports whenever the window size changes
    // so then each camera always takes up half the screen.
    // A resize_event is sent when the window is first created, allowing us to reuse this system for initial setup.
    for resize_event in resize_events.read() {
        let window = windows.get(resize_event.window).unwrap();
        if let Ok(mut left_camera) = left_camera.single_mut() {
            left_camera.viewport = Some(Viewport {
                physical_position: UVec2::new(0, 0),
                physical_size: UVec2::new(window.physical_width() / 2, window.physical_height()),
                ..default()
            });
        }

        if let Ok(mut right_camera) = right_camera.single_mut() {
            right_camera.viewport = Some(Viewport {
                physical_position: UVec2::new(window.physical_width() / 2, 0),
                physical_size: UVec2::new(window.physical_width() / 2, window.physical_height()),
                ..default()
            });
        }
    }
}

fn switch_camera(
    mut settings: ResMut<SpectatorSettings>,
    keys: Res<ButtonInput<KeyCode>>,
    left_camera: Query<Entity, (With<LeftCamera>, Without<RightCamera>)>,
    right_camera: Query<Entity, With<RightCamera>>,
) {
    let Ok(left_camera) = left_camera.single() else {
        return;
    };
    let Ok(right_camera) = right_camera.single() else {
        return;
    };

    if keys.just_pressed(KeyCode::KeyE) {
        if let Some(spectator) = settings.active_spectator {
            if spectator == left_camera {
                settings.active_spectator = Some(right_camera);
            } else {
                settings.active_spectator = Some(left_camera);
            }
        } else {
            settings.active_spectator = Some(left_camera);
        }
        println!("Switched camera");
    }
}
