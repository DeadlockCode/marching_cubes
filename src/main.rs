pub mod marching_cubes;
pub mod surface_nets;
pub mod showcase;
pub mod visualization;
pub mod circle_fan;

pub use bevy::prelude::*;

pub const WIDTH: f32 = 1280.0;
pub const HEIGHT: f32 = 720.0;

pub const MOVE_SPEED: f32 = 30.0;
pub const SENSITIVITY: f32 = 1.0;

const APP: u32 = 1;

fn main() {
    match APP {
        0 => showcase::start(),
        1 => visualization::start(),
        _ => return,
    }
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