mod march_tables;

use bevy::{prelude::*, pbr::wireframe::WireframeConfig};

fn index(x: usize, y: usize, z: usize, resolution: usize) -> usize {
    x + (y + z * resolution) * resolution
}

fn generate_mesh() -> Mesh {
    const RESOLUTION: usize = 64;

    let mut points = [0.0_f32; (RESOLUTION + 1) * (RESOLUTION + 1) * (RESOLUTION + 1)];

    let step = 1.0 / (RESOLUTION + 1) as f32;

    let floor = 0.05;
    

    for z in 0..(RESOLUTION + 1) {
        for y in 0..(RESOLUTION + 1) {
            for x in 0..(RESOLUTION + 1) {
                let idx = index(x, y, z, RESOLUTION + 1);
                let pos = Vec3::new(x as f32 * step - 0.5, y as f32 * step - 0.5, z as f32 * step - 0.5);
                points[idx] = pos.length_squared();
            }
        }
    }

    let mut positions = Vec::<[f32; 3]>::new();
    let mut indices = Vec::<u32>::new();

    let mut current_index = 0;
    for z in 0..RESOLUTION {
        for y in 0..RESOLUTION {
            for x in 0..RESOLUTION {
                
                let mut triangulation_index = 0;
                if points[index(  x  ,  y  ,  z  , RESOLUTION)] > floor { triangulation_index |= 1 << 0; };
                if points[index(  x  ,  y  ,z + 1, RESOLUTION)] > floor { triangulation_index |= 1 << 1; };
                if points[index(x + 1,  y  ,z + 1, RESOLUTION)] > floor { triangulation_index |= 1 << 2; };
                if points[index(x + 1,  y  ,  z  , RESOLUTION)] > floor { triangulation_index |= 1 << 3; };
                if points[index(  x  ,y + 1,  z  , RESOLUTION)] > floor { triangulation_index |= 1 << 4; };
                if points[index(  x  ,y + 1,z + 1, RESOLUTION)] > floor { triangulation_index |= 1 << 5; };
                if points[index(x + 1,y + 1,z + 1, RESOLUTION)] > floor { triangulation_index |= 1 << 6; };
                if points[index(x + 1,y + 1,  z  , RESOLUTION)] > floor { triangulation_index |= 1 << 7; };
                let triangulation = march_tables::triangulations[triangulation_index];

                for edge_index in triangulation {
                    if edge_index.is_negative() { break; }

                    let pos_a: Vec3 = Vec3::new(x as f32, y as f32, z as f32) + march_tables::point_offsets[march_tables::corner_index_a_from_edge[edge_index as usize]];
                    let pos_b: Vec3 = Vec3::new(x as f32, y as f32, z as f32) + march_tables::point_offsets[march_tables::corner_index_b_from_edge[edge_index as usize]];

                    let val_a: f32 = points[index(pos_a.x as usize, pos_a.y as usize, pos_a.z as usize, RESOLUTION)];
                    let val_b: f32 = points[index(pos_b.x as usize, pos_b.y as usize, pos_b.z as usize, RESOLUTION)];

                    let s = (floor - val_b) / (val_a - val_b);

                    let position = pos_b.lerp(pos_a, s);

                    let mut make_vertex = true;

                    //for i in 0..positions.len() {
                    //    let other: Vec3 = positions[i].into();
                    //    if other == (position * step - Vec3::ONE * 0.5) {
                    //        indices.push(i as u32);
                    //        make_vertex = false;
                    //    }
                    //}
                    
                    if make_vertex {
                        positions.push((position * step - Vec3::ONE * 0.5).into());
                    
                        indices.push(current_index);
                        current_index += 1;
                    }
                }
            }
        }
    }

    let mut normals = vec![[0.0, 0.0, 0.0]; positions.len() as usize];

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

    mesh
}

pub fn spawn_marching_cubed_surface(
    mut commands: Commands,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    wireframe_config.global = false;

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(generate_mesh()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
}