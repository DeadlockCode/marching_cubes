use super::*;

use crate::visualization_helper::*;

use bevy::{render::mesh::Indices, log::LogSettings};
use bevy_inspector_egui::{WorldInspectorPlugin, RegisterInspectable};

use meshtext;


fn scalar_field(i: f32, j: f32, k: f32) -> f32 {
    let mul = (128.0/17.0);

    let (x, y, z) = (i * mul - 4.0, j * mul - 4.0, k * mul - 4.0);

    (x-2.0)*(x-2.0)*(x+2.0)*(x+2.0) + (y-2.0)*(y-2.0)*(y+2.0)*(y+2.0) + (z-2.0)*(z-2.0)*(z+2.0)*(z+2.0) + 3.0*(x*x*y*y+x*x*z*z+y*y*z*z) + 6.0*x*y*z - 10.0*(x*x+y*y+z*z) + 22.0
}
fn sphere(i: f32, j: f32, k: f32) -> f32 {
    let (x, y, z) = (i - 0.5, j - 0.5, k - 0.5);

    x*x + y*y + z*z - 0.2
}

const SCALAR_FIELD: &dyn Fn(f32, f32, f32) -> f32 = &sphere;

pub fn start() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb_u8(20, 20, 20)))
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
        .add_system(camera_system)


        .add_startup_system(spawn_point_light)

        .add_startup_system(spawn_boundary_cube)

        .add_startup_system(spawn_grid_points)
        .add_system(grid_point_system)

        .add_startup_system(spawn_grid_mesh)
        .add_system(grid_mesh_system)

        .add_startup_system(spawn_mesh_holder)
        .add_system(interpolate_mesh_system)

        .add_system(look_at_camera_system)

        .register_inspectable::<Isosurface>()
        .run();
}


fn spawn_grid_points(
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

    let mut binding = commands.spawn_bundle(SpatialBundle::default());
    let grid = binding.insert(Name::new("Grid"));´

    let largest = f32::MIN;

    let points = coords(2)
        .map(|(x, y, z)| {
            let val = SCALAR_FIELD(x as f32, y as f32, z as f32);

            largest = largest.max(val);

            val
        })
        .collect::<Vec<_>>();
    let discrete_scalar_field = &move |x, y, z| {
        points[x + y * 2 + z * 4] / largest
    };

    //let mesh = meshtext::MeshGenerator::new(font)

    grid.add_children(|parent| {
        for z in 0..2 {
            for y in 0..2 {
                for x in 0..2 {
                    let col = (discrete_scalar_field(x, y, z)).max(0.0).sqrt();
    
                    parent.spawn_bundle(MaterialMeshBundle {
                        mesh: shared_mesh.clone(),
                        material: materials.add(StandardMaterial {
                            base_color: Color::rgb(col, col, col), 
                            unlit: true,
                            ..Default::default()
                        }),
                        transform: Transform::from_translation(Vec3::new(x as f32, y as f32, z as f32) - Vec3::splat(0.5)).with_scale(Vec3::splat(0.01)),
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

    println!("min: {}, max: {}", smallest, largest);
}
