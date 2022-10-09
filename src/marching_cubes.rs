pub mod march_tables;

use super::*;

use bevy::{prelude::*, utils::HashMap};
use stopwatch::Stopwatch;

type ScalarField = dyn Fn(f32, f32, f32) -> f32;
type DiscreteScalarField = dyn Fn(usize, usize, usize) -> f32;

pub fn marching_cubes(
    resolution: usize,
    scalar_field: &ScalarField,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let sw = Stopwatch::start_new();

    let axis_length = resolution + 1;
    let grid = coords(resolution + 1)
        .map(|(x, y, z)| scalar_field(x as f32, y as f32, z as f32))
        .collect::<Vec<_>>();
    let discrete_scalar_field = &move |x, y, z| {
        unsafe { *grid.get_unchecked(x + y * axis_length + z * axis_length * axis_length) }
    };

    let mut positions = Vec::<[f32; 3]>::new();
    let mut indices = Vec::<u32>::new();
    let mut edge_to_index = HashMap::<(usize, usize, usize), u32>::new();

    for z in 0..resolution {
        for y in 0..resolution {
            for x in 0..resolution {
                
                let triangulation = get_triangulation(discrete_scalar_field, (x, y, z));

                for edge_index in triangulation {
                    if edge_index == march_tables::INV { break; }

                    make_vertex(discrete_scalar_field, &mut positions, &mut edge_to_index, &mut indices, (x, y, z), edge_index as usize);
                }
            }
        }
    }

    let normals = calculate_gradient_normals(&positions, scalar_field);

    println!("Marching cubes took: {}ms", sw.elapsed_ms());

    (positions, normals, indices)
}

pub fn marching_cubes_interpolation(
    resolution: usize,
    scalar_field: &ScalarField,
    interpolate: f32,
    normal_weight: f32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>){
    let axis_length = resolution + 1;
    let grid = coords(axis_length)
        .map(|(x, y, z)| scalar_field(x as f32, y as f32, z as f32))
        .collect::<Vec<_>>();
    let discrete_scalar_field = &move |x, y, z| {
        unsafe { *grid.get_unchecked(x + y * axis_length + z * axis_length * axis_length) }
    };

    let mut positions = Vec::<[f32; 3]>::new();

    for z in 0..resolution {
        for y in 0..resolution {
            for x in 0..resolution {
                
                let triangulation = get_triangulation(discrete_scalar_field, (x, y, z));

                for edge_index in triangulation {
                    if edge_index == march_tables::INV { break; }

                    make_vertex_interpolation(discrete_scalar_field, &mut positions, (x, y, z), edge_index as usize, interpolate);
                }
            }
        }
    }

    let gradient_normals = calculate_gradient_normals(&positions, scalar_field);
    let flat_normals = calculate_flat_normals(&positions);

    let mut normals = vec![[0.0; 3]; positions.len()];

    for i in 0..positions.len() {
        let f_n = flat_normals[i];
        let g_n = gradient_normals[i];

        normals[i] = [f_n[0] + (g_n[0] - f_n[0]) * normal_weight, f_n[1] + (g_n[1] - f_n[1]) * normal_weight, f_n[2] + (g_n[2] - f_n[2]) * normal_weight];
    }

    (positions, normals)
}

pub fn marching_cubes_disjointed(
    resolution: usize,
    scalar_field: &ScalarField,
) -> Vec<Vec<[f32; 3]>> {
    let sw = Stopwatch::start_new();

    let axis_length = resolution + 1;
    let grid = coords(axis_length)
        .map(|(x, y, z)| scalar_field(x as f32, y as f32, z as f32))
        .collect::<Vec<_>>();
    let discrete_scalar_field = &move |x, y, z| {
        grid[x + y * axis_length + z * axis_length * axis_length]
    };

    let mut positions_vec = Vec::new();


    for z in 0..resolution {
        for y in 0..resolution {
            for x in 0..resolution {
                let mut positions = Vec::<[f32; 3]>::new();
                
                let triangulation = get_triangulation(discrete_scalar_field, (x, y, z));

                for edge_index in triangulation {
                    if edge_index == march_tables::INV { break; }

                    make_vertex_interpolation(discrete_scalar_field, &mut positions, (x, y, z), edge_index as usize, 0.0);
                }

                positions_vec.push(positions);
            }
        }
    }

    println!("Marching cubes took: {}ms", sw.elapsed_ms());

    positions_vec
}

fn make_vertex(
    discrete_scalar_field: &DiscreteScalarField,
    positions: &mut Vec<[f32; 3]>,
    edge_to_index: &mut HashMap<(usize, usize, usize), u32>,
    indices: &mut Vec<u32>,
    (x, y, z): (usize, usize, usize),
    edge_index: usize,
) {
    let (x0, y0, z0) = march_tables::POINT_OFFSETS[march_tables::CORNER_INDEX_A_FROM_EDGE[edge_index]];
    let (x1, y1, z1) = march_tables::POINT_OFFSETS[march_tables::CORNER_INDEX_B_FROM_EDGE[edge_index]];

    let edge = (x * 2 + x0 + x1, y * 2 + y0 + y1, z * 2 + z0 + z1);

    match edge_to_index.get(&edge) {
        Some(i) => indices.push(*i),
        None => {
            let pos_a: Vec3 = Vec3::new((x + x0) as f32, (y + y0) as f32, (z + z0) as f32);
            let pos_b: Vec3 = Vec3::new((x + x1) as f32, (y + y1) as f32, (z + z1) as f32);
        
            let val_a = discrete_scalar_field(pos_a.x as usize, pos_a.y as usize, pos_a.z as usize);
            let val_b = discrete_scalar_field(pos_b.x as usize, pos_b.y as usize, pos_b.z as usize);
        
            let t = val_a / (val_a - val_b);
        
            let position = (pos_a + (pos_b - pos_a) * t).into();
        
            indices.push(positions.len() as u32);
            edge_to_index.insert(edge, positions.len() as u32);
            positions.push(position);
        },
    }
}

fn make_vertex_interpolation(
    discrete_scalar_field: &DiscreteScalarField,
    positions: &mut Vec<[f32; 3]>,
    (x, y, z): (usize, usize, usize),
    edge_index: usize,
    interpolate: f32,
) {
    let (x0, y0, z0) = march_tables::POINT_OFFSETS[march_tables::CORNER_INDEX_A_FROM_EDGE[edge_index]];
    let (x1, y1, z1) = march_tables::POINT_OFFSETS[march_tables::CORNER_INDEX_B_FROM_EDGE[edge_index]];

    let pos_a: Vec3 = Vec3::new((x + x0) as f32, (y + y0) as f32, (z + z0) as f32);
    let pos_b: Vec3 = Vec3::new((x + x1) as f32, (y + y1) as f32, (z + z1) as f32);

    let val_a = discrete_scalar_field(pos_a.x as usize, pos_a.y as usize, pos_a.z as usize);
    let val_b = discrete_scalar_field(pos_b.x as usize, pos_b.y as usize, pos_b.z as usize);

    let t = val_a / (val_a - val_b);

    let t2 = 0.5 + (t - 0.5) * interpolate;

    let position = (pos_a + (pos_b - pos_a) * t2).into();

    positions.push(position);
}

fn get_triangulation(
    discrete_scalar_field: &DiscreteScalarField,
    (x, y, z): (usize, usize, usize),
) -> [usize; 16] {
    let mut triangulation_index = 0;

    if discrete_scalar_field(  x  ,  y  ,  z  ) > 0.0 { triangulation_index |= 1 << 0; };
    if discrete_scalar_field(  x  ,  y  ,z + 1) > 0.0 { triangulation_index |= 1 << 1; };
    if discrete_scalar_field(x + 1,  y  ,z + 1) > 0.0 { triangulation_index |= 1 << 2; };
    if discrete_scalar_field(x + 1,  y  ,  z  ) > 0.0 { triangulation_index |= 1 << 3; };
    if discrete_scalar_field(  x  ,y + 1,  z  ) > 0.0 { triangulation_index |= 1 << 4; };
    if discrete_scalar_field(  x  ,y + 1,z + 1) > 0.0 { triangulation_index |= 1 << 5; };
    if discrete_scalar_field(x + 1,y + 1,z + 1) > 0.0 { triangulation_index |= 1 << 6; };
    if discrete_scalar_field(x + 1,y + 1,  z  ) > 0.0 { triangulation_index |= 1 << 7; };
    
    march_tables::TRIANGULATIONS[triangulation_index]
}

fn gradient(x: f32, y: f32, z: f32, scalar_field: &dyn Fn(f32, f32, f32) -> f32) -> [f32; 3] {
    let val_o = scalar_field(x, y, z);
    let val_x = scalar_field(x + 0.001, y, z);
    let val_y = scalar_field(x, y + 0.001, z);
    let val_z = scalar_field(x, y, z + 0.001);
    return [val_x - val_o, val_y - val_o, val_z - val_o]
}

fn calculate_gradient_normals(
    positions: &Vec<[f32; 3]>,
    scalar_field: &dyn Fn(f32, f32, f32) -> f32,
) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0; 3]; positions.len()];

    for (i, position) in positions.iter().enumerate() {
        let normal = Into::<Vec3>::into(gradient(position[0], position[1], position[2], scalar_field)).normalize();
        normals[i] = normal.into();
    }

    normals
}

fn calculate_flat_normals(
    positions: &Vec<[f32; 3]>,
) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0; 3]; positions.len()];

    for i in 0..positions.len()/3 {
        let p1: Vec3 = positions[i * 3 + 0].into();
        let p2: Vec3 = positions[i * 3 + 1].into();
        let p3: Vec3 = positions[i * 3 + 2].into();
        
        let n = (p2 - p1).cross(p3 - p1);

        normals[i * 3 + 0] = n.normalize().into();
        normals[i * 3 + 1] = n.normalize().into();
        normals[i * 3 + 2] = n.normalize().into();
    }

    normals
}