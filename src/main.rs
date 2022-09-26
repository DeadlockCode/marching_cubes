mod marching_cubes;
mod surface_nets;

use bevy::{prelude::*, input::mouse::MouseMotion, render::{settings::{WgpuSettings, WgpuFeatures}, mesh::Indices}, pbr::wireframe::{WireframePlugin, WireframeConfig}};
use bevy_inspector_egui::WorldInspectorPlugin;

pub const WIDTH: f32 = 1280.0;
pub const HEIGHT: f32 = 720.0;

pub const MOVE_SPEED: f32 = 30.0;
pub const SENSITIVITY: f32 = 1.0;

fn main() {
    App::new()
        .insert_resource(WgpuSettings {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..default()
        })
        .insert_resource(Msaa { samples: 4 })
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
        .add_startup_system(spawn_thing)
        .add_startup_system(spawn_other_thing)
        .add_startup_system(spawn_light)
        .add_system(update_camera)
        .run();
}

fn update_camera(
    keys: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
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
        if buttons.pressed(MouseButton::Left) {
            for ev in motion_evr.iter() {
                camera.rotate_local_axis(Vec3::X, -ev.delta.y * delta * SENSITIVITY);
                camera.rotate_axis(Vec3::Y, -ev.delta.x * delta * SENSITIVITY);
            }
        }
    }
}

const RESOLUTION: usize = 32;

fn implicit_function(i: f32, j: f32, k: f32) -> f32 {
    let res = RESOLUTION as f32 * 0.5;
    let mul = 3.6 / res;

    let (x, y, z) = ((i - res) * mul, (j - res) * mul, (k - res) * mul);

    (x-2.0)*(x-2.0)*(x+2.0)*(x+2.0) + (y-2.0)*(y-2.0)*(y+2.0)*(y+2.0) + (z-2.0)*(z-2.0)*(z+2.0)*(z+2.0) + 3.0*(x*x*y*y+x*x*z*z+y*y*z*z) + 6.0*x*y*z - 10.0*(x*x+y*y+z*z) + 22.0
}

fn spawn_thing(
    mut commands: Commands,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    wireframe_config.global = true;

    let (positions, normals, indices) = surface_nets::surface_net(RESOLUTION, &implicit_function);

    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(Color::rgb(0.4, 0.7, 1.0).into()),
        transform: Transform::from_translation(Vec3::new(-(RESOLUTION as f32), 0.0, 0.0)),
        ..Default::default()
    });
}

pub fn spawn_other_thing(
    mut commands: Commands,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    wireframe_config.global = true;

    let (positions, normals, indices) = marching_cubes::marching_cubes(RESOLUTION, &implicit_function);

    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(Color::rgb(0.4, 0.7, 1.0).into()),
        ..Default::default()
    });
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
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