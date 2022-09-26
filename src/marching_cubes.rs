mod march_tables;

use bevy::prelude::*;

fn index(x: usize, y: usize, z: usize, resolution: usize) -> usize {
    x + y * resolution + z * resolution * resolution
}

pub type SDF = dyn Fn(f32, f32, f32) -> f32;

pub fn marching_cubes(
    resolution: usize,
    signed_distance_field: &SDF,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let points = coords(resolution + 1)
        .map(|(x, y, z)| signed_distance_field(x as f32, y as f32, z as f32))
        .collect::<Vec<_>>();

    let mut positions = Vec::<[f32; 3]>::new();
    let mut indices = Vec::<u32>::new();

    let mut vertex_amt = 0;
    for z in 0..resolution {
        for y in 0..resolution {
            for x in 0..resolution {
                
                let mut triangulation_index = 0;

                if points[index(  x  ,  y  ,  z  , resolution + 1)] > 0.0 { triangulation_index |= 1 << 0; };
                if points[index(  x  ,  y  ,z + 1, resolution + 1)] > 0.0 { triangulation_index |= 1 << 1; };
                if points[index(x + 1,  y  ,z + 1, resolution + 1)] > 0.0 { triangulation_index |= 1 << 2; };
                if points[index(x + 1,  y  ,  z  , resolution + 1)] > 0.0 { triangulation_index |= 1 << 3; };
                if points[index(  x  ,y + 1,  z  , resolution + 1)] > 0.0 { triangulation_index |= 1 << 4; };
                if points[index(  x  ,y + 1,z + 1, resolution + 1)] > 0.0 { triangulation_index |= 1 << 5; };
                if points[index(x + 1,y + 1,z + 1, resolution + 1)] > 0.0 { triangulation_index |= 1 << 6; };
                if points[index(x + 1,y + 1,  z  , resolution + 1)] > 0.0 { triangulation_index |= 1 << 7; };
                
                let triangulation = march_tables::TRIANGULATIONS[triangulation_index];

                for edge_index in triangulation {
                    if edge_index == -1 { break; }

                    let pos_a: Vec3 = Vec3::new(x as f32, y as f32, z as f32) + march_tables::POINT_OFFSETS[march_tables::CORNER_INDEX_A_FROM_EDGE[edge_index as usize]];
                    let pos_b: Vec3 = Vec3::new(x as f32, y as f32, z as f32) + march_tables::POINT_OFFSETS[march_tables::CORNER_INDEX_B_FROM_EDGE[edge_index as usize]];

                    let val_a = points[index(pos_a.x as usize, pos_a.y as usize, pos_a.z as usize, resolution + 1)] as f32;
                    let val_b = points[index(pos_b.x as usize, pos_b.y as usize, pos_b.z as usize, resolution + 1)] as f32;

                    let t = (-val_b) / (val_a - val_b);

                    let position = pos_b + (pos_a - pos_b) * t;
                    
                    let mut make_vertex = true;

                    for idx in 0..vertex_amt {
                        let this: [f32; 3] = position.into(); 
                        if positions[idx] == this {
                            make_vertex = false;
                            indices.push(idx as u32);
                        }
                    }

                    if make_vertex {
                        positions.push(position.into());
                        indices.push(vertex_amt as u32);
                        vertex_amt += 1;
                    }
                }
            }
        }
    }

    let mut normals = vec![[0.0; 3]; positions.len()];

    for idx in 0..indices.len()/3 {
        let norm1: Vec3 = normals[indices[idx * 3 + 0] as usize].into();
        let norm2: Vec3 = normals[indices[idx * 3 + 1] as usize].into();
        let norm3: Vec3 = normals[indices[idx * 3 + 2] as usize].into();

        let pos1: Vec3 = positions[indices[idx * 3 + 0] as usize].into();
        let pos2: Vec3 = positions[indices[idx * 3 + 1] as usize].into();
        let pos3: Vec3 = positions[indices[idx * 3 + 2] as usize].into();

        let normal = Vec3::cross((pos2 - pos1).normalize(), (pos3 - pos1).normalize());

        normals[indices[idx * 3 + 0] as usize] = (norm1 + normal).into();
        normals[indices[idx * 3 + 1] as usize] = (norm2 + normal).into();
        normals[indices[idx * 3 + 2] as usize] = (norm3 + normal).into();
    }

    for idx in 0..normals.len() {
        normals[idx] = Into::<Vec3>::into(normals[idx]).normalize().into();
    }

    (positions, normals, indices)
}

fn coords(size: usize) -> impl Iterator<Item = (usize, usize, usize)> {
    (0..size)
        .flat_map(move |x| (0..size).map(move |y| (x, y)))
        .flat_map(move |(x, y)| (0..size).map(move |z| (x, y, z)))
}