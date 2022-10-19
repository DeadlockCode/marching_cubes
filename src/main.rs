pub mod marching_cubes;
pub mod surface_nets;
pub mod showcase;
pub mod visualization_of_marching_cubes_full;
pub mod visualization_of_marching_cubes_zoom;
pub mod circle_fan;
pub mod visualization_helper;
pub mod normal_material;
pub mod cube_sphere;

use std::f32::consts::PI;

pub use bevy::prelude::*;
use bevy::render::camera::Projection;

pub const WIDTH: f32 = 1280.0;
pub const HEIGHT: f32 = 720.0;

const APP: u32 = 0;

fn main() {
    match APP {
        0 => showcase::start(),
        1 => visualization_of_marching_cubes_full::start(),
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
        transform: Transform::from_xyz(0.75, 1.0, 1.0),
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

fn camera_system (
    mut cameras: Query<(&mut Transform, With<Camera3d>)>,
    time: Res<Time>,
) {
    let mut camera = cameras.single_mut().0;

    let t = time.seconds_since_startup() as f32 * 2.0 * PI / 60.0;

    camera.translation = Vec3::new(t.cos(), 0.5, t.sin()) * 2.2;
    camera.look_at(Vec3::new(0.0, -0.1, 0.0), Vec3::Y)
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        projection: Projection::Perspective(PerspectiveProjection { fov: 0.65, ..Default::default() }),
        ..Default::default()
    })
    .insert(Name::new("Camera"));
}

fn coords(size: usize) -> impl Iterator<Item = (usize, usize, usize)> {
    (0..size).flat_map(move |z| {
        (0..size).map(move |y| (y, z))
    }).flat_map(move |(y, z)| {
        (0..size).map(move |x| (x, y, z))
    })
}