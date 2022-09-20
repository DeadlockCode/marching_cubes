use bevy::{prelude::*, input::mouse::MouseMotion, render::settings::{WgpuSettings, WgpuFeatures}, pbr::wireframe::{WireframePlugin, WireframeConfig}};
use bevy_inspector_egui::WorldInspectorPlugin;

use noise::{Fbm, NoiseFn};

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
        .add_startup_system(spawn_scene)
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
            let x = camera.local_x();
            let y = Vec3::new(0.0, 1.0, 0.0);

            camera.rotate_axis(x, -ev.delta.y * delta * SENSITIVITY);
            camera.rotate_axis(y, -ev.delta.x * delta * SENSITIVITY);
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    })
    .insert(Name::new("Camera"));
}

fn cube_to_sphere(v: Vec3) -> Vec3 {
    Vec3::new(
        v.x * (1.0 - v.y * v.y * 0.5 - v.z * v.z * 0.5 + v.y * v.y * v.z * v.z / 3.0).sqrt(),
        v.y * (1.0 - v.z * v.z * 0.5 - v.x * v.x * 0.5 + v.z * v.z * v.x * v.x / 3.0).sqrt(),
        v.z * (1.0 - v.x * v.x * 0.5 - v.y * v.y * 0.5 + v.x * v.x * v.y * v.y / 3.0).sqrt(),
    )
}

fn sphere_to_planet(v: Vec3) -> Vec3 {
    let fbm = Fbm::new();
    let val = fbm.get([v.x as f64, v.y as f64, v.z as f64]) as f32;
    v + v * (val * 0.5 - 0.05).max(0.0)
}

fn generate_cube_face(resolution: usize, local_y: Vec3) -> Mesh {
    let local_x = Vec3::new(local_y.y, local_y.z, local_y.x);
    let local_z = local_y.cross(local_x);

    let mut positions = Vec::new();
    positions.resize(resolution * resolution, [0.0, 0.0, 0.0]);
    let mut normals = Vec::new();
    normals.resize(resolution * resolution, [0.0, 0.0, 0.0]);
    let mut uvs = Vec::new();
    uvs.resize(resolution * resolution, [0.0, 0.0]);

    let mut indices = Vec::new();
    indices.resize((resolution - 1) * (resolution - 1) * 6, 0 as u32);

    for y in 0..resolution {
        for x in 0..resolution {
            let idx = x + y * resolution;
            let percent = Vec2::new(x as f32, y as f32) / (resolution - 1) as f32;
            let cube = local_y + local_x * (percent.x * 2.0 - 1.0) + local_z * (percent.y * 2.0 - 1.0);
            let sphere = cube_to_sphere(cube);
            positions[idx] = sphere_to_planet(sphere).into();

            if x != resolution - 1 && y != resolution - 1 {
                let idx_2 = (x + y * (resolution - 1)) * 6;
                indices[  idx_2  ] = (idx                 ) as u32;
                indices[idx_2 + 1] = (idx + resolution + 1) as u32;
                indices[idx_2 + 2] = (idx + resolution    ) as u32;
                indices[idx_2 + 3] = (idx                 ) as u32;
                indices[idx_2 + 4] = (idx              + 1) as u32;
                indices[idx_2 + 5] = (idx + resolution + 1) as u32;
            }
        }
    }
    
    for i in 0..(indices.len() / 3) {
        let v1: Vec3 = positions[indices[i * 3 + 0] as usize].into();
        let v2: Vec3 = positions[indices[i * 3 + 1] as usize].into();
        let v3: Vec3 = positions[indices[i * 3 + 2] as usize].into();
    
        let prev1: Vec3 = normals[indices[i * 3 + 0] as usize].into();
        let prev2: Vec3 = normals[indices[i * 3 + 1] as usize].into();
        let prev3: Vec3 = normals[indices[i * 3 + 2] as usize].into();
    
        normals[indices[i * 3 + 0] as usize] = (prev1 + (v2 - v1).cross(v3 - v1)).into();
        normals[indices[i * 3 + 1] as usize] = (prev2 + (v2 - v1).cross(v3 - v1)).into();
        normals[indices[i * 3 + 2] as usize] = (prev3 + (v2 - v1).cross(v3 - v1)).into();
    }
    
    for i in 0..normals.len() {
        let normal: Vec3 = normals[i].into();
        normals[i] = normal.normalize().into();
    }

    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    mesh
}

fn spawn_scene(
    mut commands: Commands,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    wireframe_config.global = false;

    let dirs = [
        Vec3::X,
        Vec3::Y,
        Vec3::Z,
        Vec3::NEG_X,
        Vec3::NEG_Y,
        Vec3::NEG_Z,
    ];
    commands.spawn_bundle(SpatialBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    })
    .with_children(|parent| {
        for i in 0..6 {
            parent.spawn_bundle(PbrBundle {
                mesh: meshes.add(generate_cube_face(32, dirs[i])),
                material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
                ..Default::default()
            });
        }
    })
    .insert(Name::new("Planet"));

    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    })
    .insert(Name::new("Light"));
}