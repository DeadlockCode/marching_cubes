mod march_tables;

use bevy::{prelude::*, utils::HashMap};
use stopwatch::Stopwatch;

type SDF = dyn Fn(f32, f32, f32) -> f32;
type GridSDF = dyn Fn(usize, usize, usize) -> f32;

pub fn marching_cubes(
    resolution: usize,
    signed_distance_field: &SDF,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let sw = Stopwatch::start_new();

    let axis_length = resolution + 1;
    let points = coords(resolution + 1)
        .map(|(x, y, z)| signed_distance_field(x as f32, y as f32, z as f32))
        .collect::<Vec<_>>();
    let grid_values = &move |x, y, z| {
        unsafe { *points.get_unchecked(z * axis_length * axis_length + y * axis_length + x) }
    };

    let mut edge_to_index = HashMap::<[usize; 3], u32>::new();
    let mut positions = Vec::<[f32; 3]>::new();
    let mut indices = Vec::<u32>::new();

    for z in 0..resolution {
        for y in 0..resolution {
            for x in 0..resolution {
                
                let triangulation = get_triangulation(grid_values, (x, y, z));

                for edge_index in triangulation {
                    if edge_index == march_tables::INV { break; }

                    make_vertex(grid_values, &mut positions, &mut edge_to_index, &mut indices, (x, y, z), edge_index as usize);
                }
            }
        }
    }

    let normals = calculate_smooth_normals(&positions, &indices);

    println!("Marching cubes took: {}ms", sw.elapsed_ms());
    (positions, normals, indices)
}

fn calculate_smooth_normals(
    positions: &Vec<[f32; 3]>,
    indices: &Vec<u32>,
) -> Vec<[f32; 3]> {
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

    normals
}

fn make_vertex(
    grid_values: &GridSDF,
    positions: &mut Vec<[f32; 3]>,
    edge_to_index: &mut HashMap<[usize; 3], u32>,
    indices: &mut Vec<u32>,
    coord: (usize, usize, usize),
    edge_index: usize,
) {
    let off_a = march_tables::POINT_OFFSETS[march_tables::CORNER_INDEX_A_FROM_EDGE[edge_index]];
    let off_b = march_tables::POINT_OFFSETS[march_tables::CORNER_INDEX_B_FROM_EDGE[edge_index]];

    let edge = [coord.0 * 2 + off_a.0 + off_b.0, coord.1 * 2 + off_a.1 + off_b.1, coord.2 * 2 + off_a.2 + off_b.2];

    match edge_to_index.get(&edge) {
        Some(i) => indices.push(*i),
        None => {
            let pos_a: Vec3 = Vec3::new((coord.0 + off_a.0) as f32, (coord.1 + off_a.1) as f32, (coord.2 + off_a.2) as f32);
            let pos_b: Vec3 = Vec3::new((coord.0 + off_b.0) as f32, (coord.1 + off_b.1) as f32, (coord.2 + off_b.2) as f32);
        
            let val_a = grid_values(pos_a.x as usize, pos_a.y as usize, pos_a.z as usize);
            let val_b = grid_values(pos_b.x as usize, pos_b.y as usize, pos_b.z as usize);
        
            let t = val_a / (val_a - val_b);
        
            let position = (pos_a + (pos_b - pos_a) * t).into();
        
            indices.push(positions.len() as u32);
            edge_to_index.insert(edge, positions.len() as u32);
            positions.push(position);
        },
    }
}

fn get_triangulation(
    grid_values: &GridSDF,
    (x, y, z): (usize, usize, usize),
) -> [usize; 16] {
    let mut triangulation_index = 0;

    if grid_values(  x  ,  y  ,  z  ) > 0.0 { triangulation_index |= 1 << 0; };
    if grid_values(  x  ,  y  ,z + 1) > 0.0 { triangulation_index |= 1 << 1; };
    if grid_values(x + 1,  y  ,z + 1) > 0.0 { triangulation_index |= 1 << 2; };
    if grid_values(x + 1,  y  ,  z  ) > 0.0 { triangulation_index |= 1 << 3; };
    if grid_values(  x  ,y + 1,  z  ) > 0.0 { triangulation_index |= 1 << 4; };
    if grid_values(  x  ,y + 1,z + 1) > 0.0 { triangulation_index |= 1 << 5; };
    if grid_values(x + 1,y + 1,z + 1) > 0.0 { triangulation_index |= 1 << 6; };
    if grid_values(x + 1,y + 1,  z  ) > 0.0 { triangulation_index |= 1 << 7; };
    
    march_tables::TRIANGULATIONS[triangulation_index]
}

fn coords(size: usize) -> impl Iterator<Item = (usize, usize, usize)> {
    (0..size)
        .flat_map(move |x| (0..size).map(move |y| (x, y)))
        .flat_map(move |(x, y)| (0..size).map(move |z| (x, y, z)))
}