use crate::marching_cubes::march_tables;

use super::*;

use bevy::{render::mesh::Indices, log::LogSettings};
use bevy_inspector_egui::{WorldInspectorPlugin, Inspectable, RegisterInspectable};

pub fn start() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb_linear(0.01, 0.01, 0.01)))
        .insert_resource(WindowDescriptor {
            width: WIDTH,
            height: HEIGHT,
            title: "Marching Cubes".to_string(),
            resizable: true,
            ..Default::default()
        })
        .insert_resource(LogSettings {
            level: bevy::log::Level::WARN,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())

        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_light)
        .add_startup_system(spawn_grid)
        .add_startup_system(spawn_cube)
        .add_startup_system(spawn_mesh)

        .add_system(grid_point_system)
        .add_system(grid_mesh_system)
        .add_system(camera_system)

        .register_inspectable::<Isosurface>()
        .run();
}

fn camera_system (
    mut cameras: Query<(&mut Transform, With<Camera3d>)>,
    time: Res<Time>,
) {
    let mut camera = cameras.single_mut().0;

    let t = time.seconds_since_startup() as f32 * 0.25;

    camera.translation = Vec3::new(t.cos(), 0.5, t.sin()) * 2.0;
    camera.look_at(Vec3::new(0.0, -0.1, 0.0), Vec3::Y)
}

fn spawn_light(
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

fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    })
    .insert(Name::new("Camera"));
}

fn spawn_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let shared_material = materials.add(
        StandardMaterial { 
            base_color: Color::ORANGE_RED,
            emissive: Color::BLACK,
            perceptual_roughness: 1.0,
            metallic: 0.0,
            reflectance: 0.0,
            cull_mode: None,
            ..Default::default()
        });
    
    let mut binding = commands.spawn_bundle(SpatialBundle::default());
    let entity = binding.insert(Name::new("Mesh"));

    entity.add_children(|parent| {
        let positions_vec = marching_cubes::marching_cubes_flat_disjointed(RES, &SCALAR_FIELD);

        for z in 0..RES {
            for y in 0..RES {
                for x in 0..RES {
                    let idx = x + y * RES + z * RES * RES;

                    let positions = positions_vec[idx].clone();
                
                    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                    mesh.compute_flat_normals();
                
                    parent.spawn_bundle(PbrBundle {
                        mesh: meshes.add(mesh),
                        material: shared_material.clone(),
                        transform: Transform::from_translation(-Vec3::splat(0.5)).with_scale(Vec3::splat(1.0 / RES as f32)),
                        ..Default::default()
                    }).insert(GridMesh{x, y, z})
                    .insert(Name::new("(".to_owned() + &x.to_string() + &", ".to_owned() + &y.to_string() + &", ".to_owned() + &z.to_string() + &")".to_owned()));
                }
            }
        }
    });


}

const RES: usize = 16;

#[derive(Component)]
struct GridPoint {
    x: usize,
    y: usize,
    z: usize,
}

#[derive(Component)]
struct GridMesh {
    x: usize,
    y: usize,
    z: usize,
}

fn spawn_cube(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::LineList);
    let mut positions = vec![[0f32; 3]; 24];
    let normals = vec![[0f32; 3]; 24];

    for edge in 0..12 {
        let p = march_tables::POINT_OFFSETS[march_tables::CORNER_INDEX_A_FROM_EDGE[edge]];
        let q = march_tables::POINT_OFFSETS[march_tables::CORNER_INDEX_B_FROM_EDGE[edge]];
        positions[edge * 2 + 0] = [p[0] as f32 - 0.5, p[1] as f32 - 0.5, p[2] as f32 - 0.5];
        positions[edge * 2 + 1] = [q[0] as f32 - 0.5, q[1] as f32 - 0.5, q[2] as f32 - 0.5];
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    let shared_mesh = meshes.add(mesh);
    let shared_material = materials.add(StandardMaterial {
            base_color: Color::DARK_GRAY,
            unlit: true,
            ..Default::default()
        });

    commands.spawn_bundle(PbrBundle {
        mesh: shared_mesh.clone(),
        material: shared_material.clone(),
        ..Default::default()
    }).insert(Name::new("Bounary"));

    commands.spawn_bundle(PbrBundle {
        mesh: shared_mesh.clone(),
        material: shared_material.clone(),
        transform: Transform::from_scale(Vec3::splat(1.0 / RES as f32)),
        visibility: Visibility {
            is_visible: false,
        },
        ..Default::default()
    }).insert(Name::new("Highlight"))
    .insert(Highlight);
}

fn grid_point_system(
    mut grid_points: Query<(&mut Transform, &mut Visibility, &GridPoint)>,
    cameras: Query<(&Transform, With<Camera3d>, Without<GridPoint>)>,
    isosurfaces: Query<&Isosurface>,
    time: Res<Time>,
) {
    let camera_pos = cameras.single().0.translation;
    let isosurface = isosurfaces.single();
    let sec = time.seconds_since_startup() as f32 - 2.0;

    for (mut transform, mut visibility, grid_point) in grid_points.iter_mut() {
        let value = SCALAR_FIELD(grid_point.x as f32, grid_point.y as f32, grid_point.z as f32);

        visibility.is_visible = 
            value.abs().sqrt().sqrt() * value.signum() <= isosurface.iso_level
            && (sec * ((RES + 1) * (RES + 1)) as f32) as usize > grid_point.x + grid_point.y * (RES + 1) + grid_point.z * (RES + 1) * (RES + 1);
        transform.look_at(camera_pos, Vec3::Y);
    }
}

fn grid_mesh_system(
    mut grid_meshes: Query<(&mut Visibility, &GridMesh, Without<Highlight>)>,
    mut mesh_highlight: Query<(&mut Transform, &mut Visibility, With<Highlight>, Without<GridMesh>)>,
    time: Res<Time>,
) {
    let sec = time.seconds_since_startup() as f32 - 4.0;
    let (mut highlight_transform, mut highlight_visibility, _, _) = mesh_highlight.single_mut();

    for (mut visibility, grid_mesh, _) in grid_meshes.iter_mut() {
        visibility.is_visible = (sec * (RES * RES / 4) as f32) as usize > grid_mesh.x + grid_mesh.y * RES + grid_mesh.z * RES * RES;
        let current = (sec * (RES  * RES / 4) as f32) as usize + 1 == grid_mesh.x + grid_mesh.y * RES + grid_mesh.z * RES * RES;
        if current {
            highlight_transform.translation = Vec3::new(grid_mesh.x as f32, grid_mesh.y as f32, grid_mesh.z as f32) / RES as f32 - Vec3::splat(0.5) + Vec3::splat(0.5) / RES as f32;
            highlight_visibility.is_visible = true;
        }
    }
}

#[derive(Inspectable, Component)]
struct Isosurface {
    #[inspectable(min = 0.0, max = 1.0)]
    iso_level: f32,
}

#[derive(Inspectable, Component)]
struct Highlight;

fn scalar_field(i: f32, j: f32, k: f32) -> f32 {
    let mul = (128.0/17.0) / RES as f32;

    let (x, y, z) = (i * mul - 4.0, j * mul - 4.0, k * mul - 4.0);

    ((x-2.0)*(x-2.0)*(x+2.0)*(x+2.0) + (y-2.0)*(y-2.0)*(y+2.0)*(y+2.0) + (z-2.0)*(z-2.0)*(z+2.0)*(z+2.0) + 3.0*(x*x*y*y+x*x*z*z+y*y*z*z) + 6.0*x*y*z - 10.0*(x*x+y*y+z*z) + 22.0)  / 2239.6206
}

fn sphere(i: f32, j: f32, k: f32) -> f32 {
    let mul = 1.0 / RES as f32;

    let (x, y, z) = (i * mul - 0.5, j * mul - 0.5, k * mul - 0.5);

    x*x + y*y + z*z - 0.2
}

const SCALAR_FIELD: &dyn Fn(f32, f32, f32) -> f32 = &sphere;

fn spawn_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (positions, normals, indices) = circle_fan::circle_fan(8);

    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    let shared_mesh = meshes.add(mesh);

    let mut largest = f32::MIN;
    let mut smallest = f32::MAX;

    let mut binding = commands.spawn_bundle(SpatialBundle::default());
    let grid = binding.insert(Name::new("Grid"));

    grid.add_children(|parent| {
        for z in 0..(RES + 1) {
            for y in 0..(RES + 1) {
                for x in 0..(RES + 1) {
                    let col = (SCALAR_FIELD(x as f32, y as f32, z as f32)).max(0.0).sqrt().sqrt();
    
                    largest = largest.max(col);
                    smallest = smallest.min(col);
    
                    parent.spawn_bundle(MaterialMeshBundle {
                        mesh: shared_mesh.clone(),
                        material: materials.add(StandardMaterial {
                            base_color: Color::rgb(col, col, col), 
                            unlit: true,
                            ..Default::default()
                        }),
                        transform: Transform::from_translation(Vec3::new(x as f32, y as f32, z as f32) / RES as f32 - Vec3::splat(0.5)).with_scale(Vec3::splat(0.01)),
                        ..Default::default()
                    }).insert(GridPoint{x, y, z})
                    .insert(Name::new("(".to_owned() + &x.to_string() + &", ".to_owned() + &y.to_string() + &", ".to_owned() + &z.to_string() + &")".to_owned()));
                }
            }
        }
    });


    commands.spawn().insert(Isosurface {
        iso_level: 1.0,
    }).insert(Name::new("Isosurface"));

    println!("min: {}, max: {}", smallest, largest);
}