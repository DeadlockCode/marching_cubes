mod cube_sphere;
mod dual_contouring;
mod marching_cubes;
use bevy::{prelude::*, input::mouse::MouseMotion, render::settings::{WgpuSettings, WgpuFeatures}, pbr::wireframe::WireframePlugin};
use bevy_inspector_egui::WorldInspectorPlugin;

use marching_cubes::spawn_marching_cubed_surface;

pub const WIDTH: f32 = 1280.0;
pub const HEIGHT: f32 = 720.0;

pub const MOVE_SPEED: f32 = 3.0;
pub const SENSITIVITY: f32 = 1.0;

fn main() {
    App::new()
        .insert_resource(WgpuSettings {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..default()
        })
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            width: WIDTH,
            height: HEIGHT,
            title: "Marching Cubes".to_string(),
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(WireframePlugin)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_marching_cubed_surface)
        .add_startup_system(spawn_light)
        .add_system(update_camera)
        .run();
}

fn update_camera(
    keys: Res<Input<KeyCode>>,
    mut motion_evr: EventReader<MouseMotion>,
    mut cameras: Query<(&mut Transform, &Camera3d)>,
    time: Res<Time>,
) {
    let delta = time.delta().as_secs_f32();

    for (mut camera, _) in &mut cameras {
        if keys.pressed(KeyCode::Space) {
            camera.translation.y += delta * MOVE_SPEED;
        }
        if keys.pressed(KeyCode::LControl) {
            camera.translation.y -= delta * MOVE_SPEED;
        }
        if keys.pressed(KeyCode::W) {
            let dir = camera.forward();
            camera.translation += dir * delta * MOVE_SPEED;
        }
        if keys.pressed(KeyCode::A) {
            let dir = camera.left();
            camera.translation += dir * delta * MOVE_SPEED;
        }
        if keys.pressed(KeyCode::S) {
            let dir = camera.back();
            camera.translation += dir * delta * MOVE_SPEED;
        }
        if keys.pressed(KeyCode::D) {
            let dir = camera.right();
            camera.translation += dir * delta * MOVE_SPEED;
        }

        for ev in motion_evr.iter() {
            camera.rotate_local_axis(Vec3::X, -ev.delta.y * delta * SENSITIVITY);
            camera.rotate_axis(Vec3::Y, -ev.delta.x * delta * SENSITIVITY);
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    })
    .insert(Name::new("Camera"));
}

fn spawn_light(
    mut commands: Commands,
) {
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 40000.0,
            ..Default::default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 2.0, 2.0, 0.0)),
        ..Default::default()
    })
    .insert(Name::new("Light"));
}