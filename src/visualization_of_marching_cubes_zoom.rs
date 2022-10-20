use super::*;

use crate::visualization_helper::*;

use bevy::{render::mesh::Indices, log::LogSettings};
use bevy_inspector_egui::{WorldInspectorPlugin, RegisterInspectable};

use meshtext::{self, Glyph, MeshText};


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
    let font_data = include_bytes!("../assets/SourceCodePro-Bold.ttf");
    let mut generator = meshtext::MeshGenerator::new(font_data);

    commands.spawn_bundle(SpatialBundle::default()).insert(Name::new("Cormers"))
    .with_children(|builder| {
        for z in 0..2usize {
            for y in 0..2usize {
                for x in 0..2usize {
                    let num = ('0' as u8 + (x + y * 2 + z * 4) as u8) as char;
                    println!("{}", num);
                    let glyph: MeshText = generator.generate_glyph(num, true, None).unwrap(); // error -------------------------------------
                
                    let mut positions = Vec::<[f32; 3]>::new();
                    let mut normals = Vec::<[f32; 3]>::new();
                    for i in 0..glyph.vertices.len()/3 {
                        positions.push([-glyph.vertices[i * 3 + 0], glyph.vertices[i * 3 + 1], glyph.vertices[i * 3 + 2]]);
                        normals.push(Vec3::Z.into());
                    }
                
                    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

                    builder.spawn_bundle(PbrBundle {
                        mesh: meshes.add(mesh),
                        transform: Transform::from_translation(Vec3::new(x as f32, y as f32, z as f32) - Vec3::splat(0.5)).with_scale(Vec3::splat(0.1)),
                        ..Default::default()
                    })
                    .insert(LookAtCamera)
                    .insert(Name::new("(".to_owned() + &x.to_string() + &", ".to_owned() + &y.to_string() + &", ".to_owned() + &z.to_string() + &")".to_owned()));
                }
            }
        }
    });
}
