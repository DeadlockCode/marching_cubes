pub mod marching_cubes;
pub mod surface_nets;
pub mod showcase;
pub mod visualization_of_marching_cubes_full;
pub mod visualization_of_marching_cubes_zoom;
pub mod circle_fan;
pub mod visualization_helper;
pub mod normal_material;
pub mod cube_sphere;

use std::{f32::consts::TAU, env};

pub use bevy::prelude::*;
use bevy::render::{camera::{Projection, DepthCalculation, CameraProjection, ComputedCameraValues}, primitives::Frustum};

pub const WIDTH: f32 = 1280.0;
pub const HEIGHT: f32 = 720.0;

const APP: u32 = 2;

fn main() {
    let args: Vec<String> = env::args().collect();
    let app = args[1].chars().next().unwrap() as u8 - '0' as u8;

    match app {
        0 => showcase::start(),
        1 => visualization_of_marching_cubes_full::start(),
        2 => visualization_of_marching_cubes_zoom::start(),
        _ => return,
    }
}

fn spawn_point_light(
    mut commands: Commands,
) {
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            color: Color::WHITE,
            intensity: 50.0,
            range: 5.0,
            radius: 5.0,
            ..Default::default()
        },
        transform: Transform::from_xyz(-1.0, 1.0, 1.0),
        ..Default::default()
    })
    .insert(Name::new("Light"));
}

fn spawn_directional_light(
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

fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera3dBundle {
        camera: Camera {
            depth_calculation: DepthCalculation::ZDifference,
            ..Default::default()
        },
        projection: Projection::Perspective(PerspectiveProjection { 
            fov: 0.65,
            ..Default::default()
        }),
        transform: Transform::from_scale(Vec3::new(-1.0, 1.0, 1.0)),
        ..Default::default()
    })
    .insert(Name::new("Camera"));
}