use crate::{normal_material::NormalMaterial, cube_sphere::spawn_cube_sphere};

use super::*;

use bevy::{input::mouse::MouseMotion, render::{settings::{WgpuSettings, WgpuFeatures}, mesh::Indices}, pbr::wireframe::{WireframePlugin, WireframeConfig}, log::LogSettings};
use bevy_inspector_egui::WorldInspectorPlugin;
use noise::{NoiseFn, Perlin, Fbm};

pub const MOVE_SPEED: f32 = 30.0;
pub const SENSITIVITY: f32 = 1.0;

pub fn start() {
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
        .insert_resource(LogSettings {
            level: bevy::log::Level::WARN,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(WireframePlugin)

        .add_plugin(MaterialPlugin::<NormalMaterial>::default())

        .add_startup_system(spawn_camera)
        //.add_startup_system(surface_nets_mesh)
        .add_startup_system(marching_cubes_mesh)

        .add_startup_system(spawn_directional_light)
        .add_system(update_camera)
        //.add_system(update_surface_nets)
        .run();
}


fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    })
    .insert(Name::new("Camera"));
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
        
            //figure out how to get time into here. // yay i did it

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

const CHUNK_RES: usize = 8;
const RES: usize = 32;

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

                    let scalar_field = move |x: f32, y: f32, z: f32| -> f32 {
                        let scale = 1.0 / RES as f64;
                        noise_func.get([x as f64 * scale + cx as f64, y as f64 * scale + cy as f64, z as f64 * scale + cz as f64]) as f32
                    };

                    let (positions, normals, indices) = marching_cubes::marching_cubes(RES, &scalar_field);
    
                    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
                    mesh.set_indices(Some(Indices::U32(indices)));
                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    
                    builder.spawn_bundle(MaterialMeshBundle {
                        mesh: meshes.add(mesh),
                        material: materials.add(NormalMaterial{}),
                        transform: Transform::from_translation(Vec3::new(cx as f32, cy as f32, cz as f32) * RES as f32 - Vec3::splat(0.5 * (CHUNK_RES * RES) as f32)),
                        ..Default::default()
                    });
                }
            }
        }
    });



}