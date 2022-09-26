mod march_tables;

use bevy::{prelude::*, pbr::wireframe::WireframeConfig};

fn index(x: usize, y: usize, z: usize, resolution: usize) -> usize {
    x + y * resolution + z * resolution * resolution
}

fn implicit_function(x: f32, y: f32, z: f32) -> f32 {
    (x-2.0)*(x-2.0)*(x+2.0)*(x+2.0) + (y-2.0)*(y-2.0)*(y+2.0)*(y+2.0) + (z-2.0)*(z-2.0)*(z+2.0)*(z+2.0) + 3.0*(x*x*y*y+x*x*z*z+y*y*z*z) + 6.0*x*y*z - 10.0*(x*x+y*y+z*z) + 22.0
}

fn generate_mesh(origin: Vec3) -> Mesh {
    const RESOLUTION: usize = 15;

    let mut points = Box::new([0.0f32; (RESOLUTION + 1) * (RESOLUTION + 1) * (RESOLUTION + 1)]);

    let step = 1.0 / RESOLUTION as f32;

    //let fbm = Fbm::new();
    for z in 0..(RESOLUTION + 1) {
        for y in 0..(RESOLUTION + 1) {
            for x in 0..(RESOLUTION + 1) {
                let idx = index(x, y, z, RESOLUTION + 1);
                points[idx] = implicit_function(x as f32 * step + origin.x, y as f32 * step + origin.y, z as f32 * step + origin.z);
                
                //((y as f64 * step as f64 + origin.y as f64) * 255.0 - fbm.get([x as f64 * step as f64 + origin.x as f64, z as f64 * step as f64 + origin.z as f64]) * 128.0).clamp(0.0, 255.0) as u8;
            }
        }
    }

    let mut positions = Vec::<[f32; 3]>::new();

    for z in 0..RESOLUTION {
        for y in 0..RESOLUTION {
            for x in 0..RESOLUTION {
                
                let mut triangulation_index = 0;

                if points[index(  x  ,  y  ,  z  , RESOLUTION + 1)] > 0.0 { triangulation_index |= 1 << 0; };
                if points[index(  x  ,  y  ,z + 1, RESOLUTION + 1)] > 0.0 { triangulation_index |= 1 << 1; };
                if points[index(x + 1,  y  ,z + 1, RESOLUTION + 1)] > 0.0 { triangulation_index |= 1 << 2; };
                if points[index(x + 1,  y  ,  z  , RESOLUTION + 1)] > 0.0 { triangulation_index |= 1 << 3; };
                if points[index(  x  ,y + 1,  z  , RESOLUTION + 1)] > 0.0 { triangulation_index |= 1 << 4; };
                if points[index(  x  ,y + 1,z + 1, RESOLUTION + 1)] > 0.0 { triangulation_index |= 1 << 5; };
                if points[index(x + 1,y + 1,z + 1, RESOLUTION + 1)] > 0.0 { triangulation_index |= 1 << 6; };
                if points[index(x + 1,y + 1,  z  , RESOLUTION + 1)] > 0.0 { triangulation_index |= 1 << 7; };
                
                let triangulation = march_tables::TRIANGULATIONS[triangulation_index];

                for edge_index in triangulation {
                    if edge_index == -1 { break; }

                    let pos_a: Vec3 = Vec3::new(x as f32, y as f32, z as f32) + march_tables::POINT_OFFSETS[march_tables::CORNER_INDEX_A_FROM_EDGE[edge_index as usize]];
                    let pos_b: Vec3 = Vec3::new(x as f32, y as f32, z as f32) + march_tables::POINT_OFFSETS[march_tables::CORNER_INDEX_B_FROM_EDGE[edge_index as usize]];

                    let val_a = points[index(pos_a.x as usize, pos_a.y as usize, pos_a.z as usize, RESOLUTION + 1)] as f32;
                    let val_b = points[index(pos_b.x as usize, pos_b.y as usize, pos_b.z as usize, RESOLUTION + 1)] as f32;

                    let t = (-val_b) / (val_a - val_b);

                    let position = pos_b + (pos_a - pos_b) * t;
                    
                    positions.push((position * step - Vec3::splat(0.5)).into());
                }
            }
        }
    }

    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.compute_flat_normals();

    mesh
}

pub fn spawn_marching_cubed_surface(
    mut commands: Commands,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    wireframe_config.global = true;

    const RESOLUTION: i32 = 8;

    let mut parent = commands.spawn_bundle(SpatialBundle {
        transform: Transform::from_scale(Vec3::splat(32.0)),
        ..Default::default()
    });

    let ofst = RESOLUTION as f32 / 2.0;

    parent.with_children(|parent| {
        for z in 0..RESOLUTION {
            for y in 0..RESOLUTION {
                for x in 0..RESOLUTION {
                    parent.spawn_bundle(PbrBundle {
                        mesh: meshes.add(generate_mesh(Vec3::new(x as f32 - ofst, y as f32 - ofst, z as f32 - ofst))),
                        material: materials.add(Color::rgb(0.4, 0.7, 1.0).into()),
                        transform: Transform::from_translation(Vec3::new(x as f32 - ofst, y as f32 - ofst, z as f32 - ofst)),
                        ..Default::default()
                    });
                }
            }
        }
    });
}