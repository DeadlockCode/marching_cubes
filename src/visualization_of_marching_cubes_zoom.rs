use super::*;

use crate::{visualization_helper::*, marching_cubes::march_tables};

use bevy::{render::{mesh::Indices, render_resource::Face}, log::LogSettings};
use bevy_inspector_egui::{WorldInspectorPlugin, RegisterInspectable};

use ttf2mesh::Value;


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

        .add_startup_system(spawn_corner_numbers)
        //.add_system(grid_point_system)

        //.add_startup_system(spawn_grid_mesh)
        //.add_system(grid_mesh_system)

        //.add_startup_system(spawn_mesh_holder)
        //.add_system(interpolate_mesh_system)

        .add_system(look_at_camera_system)

        //.register_inspectable::<Isosurface>()
        .run();
}


fn spawn_corner_numbers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut font = ttf2mesh::TTFFile::from_file("C:\\Projects\\marching_cubes\\assets\\RobotoMono-Regular.ttf").unwrap();

    let shared_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        ..Default::default()
    });

    commands.spawn_bundle(SpatialBundle::default())
    .insert(Name::new("Cormers"))
    .with_children(|builder| {
        for z in 0..2usize {
            for y in 0..2usize {
                for x in 0..2usize {
                    const INDEX: [usize; 8] = [
                        1,
                        2,
                        5,
                        6,
                        0,
                        3,
                        4,
                        7,
                    ];

                    let num = ('0' as u8 + (INDEX[x + y * 2 + z * 4]) as u8) as char;
                    println!("{}", num);

                    let mut glyph = font.glyph_from_char(num).unwrap();

                    let bad_mesh = glyph.to_2d_mesh(ttf2mesh::Quality::High).unwrap();

                    let positions = bad_mesh.iter_vertices()
                        .map(|v| {
                            let v = v.val();
                            [-v.0 + 0.3, v.1 - 0.3, 0.0]
                        })
                        .collect::<Vec<_>>();

                    let mut indices = Vec::<u32>::new();

                    bad_mesh.iter_faces()
                        .for_each(|f| {
                            let f = f.val();
                            indices.push(f.0 as u32);
                            indices.push(f.1 as u32);
                            indices.push(f.2 as u32);
                        });

                    let normals = vec![[0.0, 0.0, -1.0]; positions.len()];
                
                    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
                    mesh.set_indices(Some(Indices::U32(indices)));
                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

                    builder.spawn_bundle(PbrBundle {
                        mesh: meshes.add(mesh),
                        transform: Transform::from_translation(Vec3::new(x as f32, y as f32, z as f32) - Vec3::splat(0.5)).with_scale(Vec3::splat(0.1)),
                        material: shared_material.clone(),
                        ..Default::default()
                    })
                    .insert(LookAtCamera)
                    .insert(Name::new("(".to_owned() + &x.to_string() + &", ".to_owned() + &y.to_string() + &", ".to_owned() + &z.to_string() + &")".to_owned()));
                }
            }
        }
    });
}
