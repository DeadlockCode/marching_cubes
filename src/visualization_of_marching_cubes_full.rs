use crate::marching_cubes::march_tables;

use super::*;

use bevy::{render::{mesh::Indices, once_cell::sync::Lazy}, log::LogSettings, window::WindowMode};
use bevy_inspector_egui::{WorldInspectorPlugin, Inspectable, RegisterInspectable};

use crate::visualization_helper::*;

const RES: usize = 12;

enum TimeStage {
    ShowGridPoints,
    SkimGridPoints,
    ShowGridMeshes,
    InterpolateMesh,
    NormalizeMesh,
}

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

#[derive(Inspectable, Component)]
struct Isosurface {
    #[inspectable()]
    isolevel: f32,
    max_value: f32,
}

#[derive(Inspectable, Component)]
struct Highlight;

fn shape(i: f32, j: f32, k: f32) -> f32 {
    let scale = (128.0/17.0) / RES as f32;

    let (x, y, z) = (i * scale - 4.0, j * scale - 4.0, k * scale - 4.0);

    (x-2.0)*(x-2.0)*(x+2.0)*(x+2.0) + (y-2.0)*(y-2.0)*(y+2.0)*(y+2.0) + (z-2.0)*(z-2.0)*(z+2.0)*(z+2.0) + 3.0*(x*x*y*y+x*x*z*z+y*y*z*z) + 6.0*x*y*z - 10.0*(x*x+y*y+z*z) + 22.0
}

fn sphere(i: f32, j: f32, k: f32) -> f32 {
    let scale = 1.0 / RES as f32;

    let (x, y, z) = (i * scale - 0.5, j * scale - 0.5, k * scale - 0.5);

    x*x + y*y + z*z - 0.2
}

const SCALAR_FIELD: &dyn Fn(f32, f32, f32) -> f32 = &sphere;

pub fn start() {
    App::new()
        .insert_resource(Timings {
            timings:  vec![10.0, 4.0, 124.0, 2.0, 0.5],
            delays:     vec![85.0, 12.0, 60.0, 5.0],
            start: -20.0
        })

        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb_u8(20, 20, 20)))
        .insert_resource(WindowDescriptor {
            mode: WindowMode::Fullscreen,
            ..Default::default()
        })
        .insert_resource(LogSettings {
            level: bevy::log::Level::WARN,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())

        .add_startup_system(spawn_camera)
        .add_system(camera_system)


        .add_startup_system(spawn_point_light)

        .add_startup_system(spawn_boundary_cube)
        .add_startup_system(spawn_mesh_cube)

        .add_startup_system(spawn_grid_points)
        .add_system(grid_point_system)

        .add_startup_system(spawn_grid_mesh)
        .add_system(grid_mesh_system)

        .add_startup_system(spawn_mesh_holder)
        .add_system(interpolate_mesh_system)

        .add_system(isolevel_system)

        .add_system(look_at_camera_system)

        .register_inspectable::<Isosurface>()
        .run();
}

fn spawn_mesh_cube(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::LineList);
    let mut positions = vec![[0f32; 3]; 24];
    let normals = vec![[0f32; 3]; 24];

    for edge_index in 0..12 {
        let point_index = march_tables::EDGES[edge_index];

        let (x0, y0, z0) = march_tables::POINTS[point_index.0];
        let (x1, y1, z1) = march_tables::POINTS[point_index.1];
        positions[edge_index * 2 + 0] = [x0 as f32, y0 as f32, z0 as f32];
        positions[edge_index * 2 + 1] = [x1 as f32, y1 as f32, z1 as f32];
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(StandardMaterial {
            base_color: Color::DARK_GRAY,
            unlit: true,
            ..Default::default()
        }),
        transform: Transform::from_scale(Vec3::splat(1.0 / RES as f32)),
        visibility: Visibility {
            is_visible: false,
        },
        ..Default::default()
    }).insert(Name::new("Highlight"))
    .insert(Highlight);
}

fn spawn_grid_points(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (positions, normals, indices) = circle_fan::circle_fan(12);

    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    let shared_mesh = meshes.add(mesh);

    let mut largest = f32::MIN;

    const GRID_RES: usize = RES + 1;

    let mut points = Vec::<f32>::with_capacity(GRID_RES * GRID_RES * GRID_RES);

    for z in 0..GRID_RES {
        for y in 0..GRID_RES {
            for x in 0..GRID_RES {
                let val = SCALAR_FIELD(x as f32, y as f32, z as f32);
                largest = largest.max(val);
                points.push(val);
            }
        }
    }

    let discrete_scalar_field = &move |x, y, z| -> f32 {
        points[x + y * (RES + 1) + z * (RES + 1) * (RES + 1)] / largest
    };

    commands.spawn_bundle(
        SpatialBundle::default()
    ).insert(Name::new("Points"))
    .with_children(|parent| {
        for z in 0..(RES + 1) {
            for y in 0..(RES + 1) {
                for x in 0..(RES + 1) {
                    let col = (discrete_scalar_field(x, y, z)).max(0.0).sqrt();
    
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
                    .insert(LookAtCamera)
                    .insert(Name::new("(".to_owned() + &x.to_string() + &", ".to_owned() + &y.to_string() + &", ".to_owned() + &z.to_string() + &")".to_owned()));
                }
            }
        }
    });

    commands.spawn().insert(Isosurface {
        isolevel: 1.0,
        max_value: largest,
    }).insert(Name::new("Isosurface"));
}

fn isolevel_system(
    mut isosurfaces: Query<&mut Isosurface>,
    time: Res<Time>,
    timings: Res<Timings>,
) {
    let t0 = timings.get_time_in_stage(TimeStage::SkimGridPoints as usize, time.seconds_since_startup() as f32);
    let t1 = timings.get_time_in_stage(TimeStage::InterpolateMesh as usize, time.seconds_since_startup() as f32);

    let mut isosurface = isosurfaces.single_mut();

    if t1 > 0.0 {
        isosurface.isolevel = -1.0
    }
    else {
        isosurface.isolevel = 1.0 - smoothstep(t0);
    }
}

fn grid_point_system(
    mut grid_points: Query<(&mut Visibility, &GridPoint)>,
    isosurfaces: Query<&Isosurface>,
    time: Res<Time>,
    timings: Res<Timings>,
) {
    let isosurface = isosurfaces.single();
    let t = timings.get_time_in_stage(TimeStage::ShowGridPoints as usize, time.seconds_since_startup() as f32);

    for (mut visibility, grid_point) in grid_points.iter_mut() {
        let value = SCALAR_FIELD(grid_point.x as f32, grid_point.y as f32, grid_point.z as f32) / isosurface.max_value;

        visibility.is_visible = 
            value.abs().sqrt() * value.signum() <= isosurface.isolevel
            && (smoothstep(t) * ((RES + 1) * (RES + 1) * (RES + 1)) as f32) as usize > grid_point.x + grid_point.y * (RES + 1) + grid_point.z * (RES + 1) * (RES + 1);
    }
}

fn spawn_grid_mesh(
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
            double_sided: true,
            cull_mode: None,
            ..Default::default()
        });
    
    let mut binding = commands.spawn_bundle(SpatialBundle::default());
    let entity = binding.insert(Name::new("Mesh"));

    entity.add_children(|parent| {
        let positions_vec = marching_cubes::marching_cubes_disjointed(RES, &SCALAR_FIELD);

        for z in 0..RES {
            for y in 0..RES {
                for x in 0..RES {
                    let idx = x + y * RES + z * RES * RES;

                    let positions = positions_vec[idx].clone();

                    let normals: Vec<[f32; 3]> = positions.chunks_exact(3).map(|p| {
                        let p1: Vec3 = p[0].into();
                        let p2: Vec3 = p[1].into();
                        let p3: Vec3 = p[2].into();

                        (p2 - p1).cross(p1 - p3).into()
                    }).flat_map(|n| [n; 3])
                    .collect();
                
                    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
                
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

fn grid_mesh_system(
    mut grid_meshes: Query<(&mut Visibility, &GridMesh), Without<Highlight>>,
    mut mesh_highlight: Query<(&mut Transform, &mut Visibility), With<Highlight>>,
    time: Res<Time>,
    timings: Res<Timings>,
) {
    let t0 = timings.get_time_in_stage(TimeStage::ShowGridMeshes as usize, time.seconds_since_startup() as f32);
    let t1 = timings.get_time_in_stage(TimeStage::InterpolateMesh as usize, time.seconds_since_startup() as f32);

    let (mut highlight_transform, mut highlight_visibility) = mesh_highlight.single_mut();

    let modified_t = (t0.powi(4) + t0) / 2.0;

    for (mut visibility, grid_mesh) in grid_meshes.iter_mut() {
        let current = (modified_t * (RES * RES * RES) as f32) as usize;

        visibility.is_visible = 
            current > grid_mesh.x + grid_mesh.y * RES + grid_mesh.z * RES * RES
            && t1 == 0.0;

        if current == grid_mesh.x + grid_mesh.y * RES + grid_mesh.z * RES * RES + 1 {
            highlight_transform.translation = Vec3::new(grid_mesh.x as f32, grid_mesh.y as f32, grid_mesh.z as f32) / RES as f32 - Vec3::splat(0.5);
            highlight_visibility.is_visible = true;
        }
        else if t0 == 1.0 {
            highlight_visibility.is_visible = false;
        }
    }
}

fn spawn_mesh_holder(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(
            StandardMaterial { 
                base_color: Color::ORANGE_RED,
                emissive: Color::BLACK,
                perceptual_roughness: 1.0,
                metallic: 0.0,
                reflectance: 0.0,
                ..Default::default()
            }),
        transform: Transform::from_translation(-Vec3::splat(0.5)).with_scale(Vec3::splat(1.0 / RES as f32)),
        visibility: Visibility{ is_visible: false },
        ..Default::default()
    }).insert(MeshHolder)
    .insert(Name::new("Mesh"));
}

fn interpolate_mesh_system(
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_holders: Query<(&mut Visibility, &Handle<Mesh>), With<MeshHolder>>,
    time: Res<Time>,
    timings: Res<Timings>,
) {
    let t0 = timings.get_time_in_stage(TimeStage::InterpolateMesh as usize, time.seconds_since_startup() as f32);
    let t1 = timings.get_time_in_stage(TimeStage::NormalizeMesh as usize, time.seconds_since_startup() as f32);

    let (positions, normals) = 
        marching_cubes::marching_cubes_interpolation(
            RES, 
            SCALAR_FIELD, 
            smoothstep(t0), 
            smoothstep(t1),
        );

    let (mut visibility, mesh_handle) = mesh_holders.single_mut();

    if t0 > 0.0 {
        visibility.is_visible = true;

        let mesh = meshes.get_mut(mesh_handle).unwrap();

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    }
}

fn camera_system (
    mut cameras: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
    timings: Res<Timings>,
) {
    let t = (time.seconds_since_startup() as f32 + timings.start) * TAU / 60.0;
    
    let mut camera = cameras.single_mut();

    camera.translation = Vec3::new(-t.sin() * 2.2, 1.0, t.cos() * 2.2);
    camera.look_at(Vec3::new(0.0, -0.15, 0.0), Vec3::Y);
}