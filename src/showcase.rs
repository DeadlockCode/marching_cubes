use crate::normal_material::NormalMaterial;

use super::*;


use bevy::{input::mouse::MouseMotion, render::{settings::{WgpuSettings, WgpuFeatures}, mesh::Indices}, pbr::wireframe::{WireframePlugin, WireframeConfig}, log::LogSettings, window::WindowMode};
use noise::{NoiseFn, Perlin, Fbm};
use stopwatch::Stopwatch;

pub const MOVE_SPEED: f32 = 30.0;
pub const SENSITIVITY: f32 = 1.0;

pub fn start() {
    App::new()
        .insert_resource(WgpuSettings {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..default()
        })
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb_linear(0.37, 1.0, 0.73)))
        .insert_resource(WindowDescriptor {
            mode: WindowMode::Fullscreen,
            title: "Marching Cubes".to_string(),
            ..Default::default()
        })
        .insert_resource(LogSettings {
            level: bevy::log::Level::WARN,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(WireframePlugin)

        .add_plugin(MaterialPlugin::<NormalMaterial>::default())

        .add_startup_system(spawn_camera)
        //.add_startup_system(surface_nets_mesh)
        .add_startup_system(marching_cubes_mesh)

        .add_startup_system(spawn_directional_light)
        .add_system(update_camera)

        //.add_system(update_surface_nets)

        .add_system(cursor_grab_system)

        .run();
}


fn cursor_grab_system(
    mut windows: ResMut<Windows>,
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
) {
    let window = windows.get_primary_mut().unwrap();

    if btn.just_pressed(MouseButton::Left) {
        window.set_cursor_lock_mode(true);
        window.set_cursor_visibility(false);
    }

    if key.just_pressed(KeyCode::Escape) {
        window.set_cursor_lock_mode(false);
        window.set_cursor_visibility(true);
    }
}

fn update_camera(
    keys: Res<Input<KeyCode>>,
    mut motion_evr: EventReader<MouseMotion>,
    mut q_camera: Query<&mut Transform, &Camera>,
    time: Res<Time>,
) {
    let delta = time.delta().as_secs_f32();

    let mut camera_transform = q_camera.single_mut();

    if keys.pressed(KeyCode::Space) {
        camera_transform.translation.y += delta * MOVE_SPEED;
    }
    if keys.pressed(KeyCode::LControl) {
        camera_transform.translation.y -= delta * MOVE_SPEED;
    }
    if keys.pressed(KeyCode::W) {
        let dir = camera_transform.forward();
        camera_transform.translation += dir * delta * MOVE_SPEED;
    }
    if keys.pressed(KeyCode::A) {
        let dir = -camera_transform.left(); // negative because camera x scale is negative to create left-handed coordinate system
        camera_transform.translation += dir * delta * MOVE_SPEED;
    }
    if keys.pressed(KeyCode::S) {
        let dir = camera_transform.back();
        camera_transform.translation += dir * delta * MOVE_SPEED;
    }
    if keys.pressed(KeyCode::D) {
        let dir = -camera_transform.right(); // negative because camera x scale is negative to create left-handed coordinate system
        camera_transform.translation += dir * delta * MOVE_SPEED;
    }
    for ev in motion_evr.iter() {
        camera_transform.rotate_local_axis(Vec3::X, -ev.delta.y * delta * SENSITIVITY);
        camera_transform.rotate_axis(Vec3::Y, ev.delta.x * delta * SENSITIVITY);
    }
}

fn update_surface_nets(
    mut surface_nets_query: Query<(&SurfaceNets, &Handle<Mesh>, &mut Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time>,
) {
    for (_, mesh_handle, mut transform) in surface_nets_query.iter_mut() {
        let mesh = meshes.get_mut(mesh_handle).unwrap();

        let my_time = (time.seconds_since_startup().cos() as f32 + 1.0) * 16.0;

        let resolution = RES - my_time as usize;

        transform.scale = Vec3::splat(32.0 / resolution as f32);


        let implicit_function = &move |i, j, k| {
            let res = resolution as f32 * 0.5;
            let mul = 3.7 / res;
        
            let (x, y, z) = ((i - res) * mul, (j - res) * mul, (k - res) * mul);

            (x-2.0)*(x-2.0)*(x+2.0)*(x+2.0)
                + (y-2.0)*(y-2.0)*(y+2.0)*(y+2.0) 
                + (z-2.0)*(z-2.0)*(z+2.0)*(z+2.0) 
                + 3.0*(x*x*y*y+x*x*z*z+y*y*z*z) 
                + 6.0*x*y*z 
                - 10.0*(x*x+y*y+z*z) 
                + 22.0
        };

        let (positions, normals, indices) = surface_nets::surface_net(resolution, implicit_function);
    
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    }  
}

#[derive(Component)]
struct SurfaceNets;

const CHUNK_RES: usize = 16;
const RES: usize = 64;

fn implicit_function(i: f32, j: f32, k: f32) -> f32 {
    let mul = (128.0/17.0) / RES as f32;

    let (x, y, z) = (i * mul - 4.0, j * mul - 4.0, k * mul - 4.0);

    (x-2.0)*(x-2.0)*(x+2.0)*(x+2.0) + (y-2.0)*(y-2.0)*(y+2.0)*(y+2.0) + (z-2.0)*(z-2.0)*(z+2.0)*(z+2.0) + 3.0*(x*x*y*y+x*x*z*z+y*y*z*z) + 6.0*x*y*z - 10.0*(x*x+y*y+z*z) + 22.0
}

fn surface_nets_mesh(
    mut commands: Commands,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    wireframe_config.global = true;

    let (positions, normals, indices) = surface_nets::surface_net(RES, &implicit_function);

    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    commands.spawn_bundle(MaterialMeshBundle {
        mesh: meshes.add(mesh),
        material: materials.add(Color::rgb(0.4, 0.7, 1.0).into()),
        transform: Transform::from_translation(Vec3::new(-(RES as f32), 0.0, 0.0)),
        ..Default::default()
    }).insert(SurfaceNets);
}

pub fn marching_cubes_mesh(
    mut commands: Commands,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<NormalMaterial>>,
) {
    wireframe_config.global = false;

    let noise_func = Perlin::default();

    commands.spawn_bundle(SpatialBundle {
        ..Default::default()
    }).insert(Name::new("World"))
    .with_children(|builder| {
        for cz in 0..CHUNK_RES {
            for cy in 0..CHUNK_RES {
                for cx in 0..CHUNK_RES {

                    let scalar_field = move |i: f32, j: f32, k: f32| -> f32 {
                        let scale = 1.0 / RES as f32;
                        let (x, y, z) = (
                            (i) * scale + cx as f32 - CHUNK_RES as f32 * 0.5,
                            (j) * scale + cy as f32 - CHUNK_RES as f32 * 0.5,
                            (k) * scale + cz as f32 - CHUNK_RES as f32 * 0.5,
                        );
                        let noise = noise_func.get([x as f64, y as f64, z as f64]) as f32;

                        noise
                    };

                    let sw = Stopwatch::start_new();
                    let (positions, normals, indices) = marching_cubes::marching_cubes(RES, &scalar_field);
                    println!("{} / {}: Marching cubes took: {}ms", cx + cy * CHUNK_RES + cz * CHUNK_RES * CHUNK_RES + 1, CHUNK_RES * CHUNK_RES * CHUNK_RES, sw.elapsed_ms());
    
                    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
                    mesh.set_indices(Some(Indices::U32(indices)));
                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

                    let position = Vec3::new(cx as f32, cy as f32, cz as f32) * RES as f32 - Vec3::splat(0.5 * (CHUNK_RES * RES) as f32);
                    let scale = Vec3::ONE;
    
                    builder.spawn_bundle(MaterialMeshBundle {
                        mesh: meshes.add(mesh),
                        material: materials.add(NormalMaterial{}),
                        transform: Transform::from_translation(position).with_scale(scale),
                        ..Default::default()
                    });
                }
            }
        }
    });



}