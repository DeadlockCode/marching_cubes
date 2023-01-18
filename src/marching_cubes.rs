pub mod march_tables;
use march_tables::{POINTS, EDGES, TRIANGULATIONS};

use bevy::{prelude::*, utils::HashMap};

type ScalarField = dyn Fn(f32, f32, f32) -> f32;
type VoxelGrid = dyn Fn(usize, usize, usize) -> f32;

pub fn marching_cubes(
    mut resolution: usize,
    scalar_field: &ScalarField,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    resolution += 1; // Cube-res to Grid-res.

    let mut voxel_grid = Vec::with_capacity(resolution * resolution * resolution);

    for z in 0..resolution {
        for y in 0..resolution {
            for x in 0..resolution {
                voxel_grid.push(scalar_field(x as f32, y as f32, z as f32));
            }
        }
    }

    let voxel_grid = move |x, y, z| {
        return voxel_grid[x + y * resolution + z * resolution * resolution];
    };
    


    let mut positions = Vec::<[f32; 3]>::new();
    let mut indices = Vec::<u32>::new();
    let mut edge_to_index = HashMap::<(usize, usize, usize), u32>::new();

    for z in 0..(resolution - 1) {
        for y in 0..(resolution - 1) {
            for x in 0..(resolution - 1) {
                march_cube(
                    (x, y, z), 
                    &voxel_grid, 
                    &mut positions, 
                    &mut indices, 
                    &mut edge_to_index,
                );
            }
        }
    }

    let normals = calculate_smooth_normals(&positions, &indices);

    (positions, normals, indices)
}

fn march_cube(
    (x, y, z): (usize, usize, usize),
    voxel_grid: &VoxelGrid,
    positions: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    edge_to_index: &mut HashMap<(usize, usize, usize), u32>,
) {
    let triangulation = get_triangulation(voxel_grid, (x, y, z));

    for edge_index in triangulation {
        if edge_index.is_negative() { break; }

        let edge = EDGES[edge_index as usize];

        let (x0, y0, z0) = POINTS[edge.0];
        let (x1, y1, z1) = POINTS[edge.1];
    
        let edge_identifier = (x * 2 + x0 + x1, y * 2 + y0 + y1, z * 2 + z0 + z1);
    
        match edge_to_index.get(&edge_identifier) {
            Some(i) => indices.push(*i),
            None => {
                let pos_a = Vec3::new((x + x0) as f32, (y + y0) as f32, (z + z0) as f32);
                let pos_b = Vec3::new((x + x1) as f32, (y + y1) as f32, (z + z1) as f32);
            
                let val_a = voxel_grid(x + x0, y + y0, z + z0);
                let val_b = voxel_grid(x + x1, y + y1, z + z1);
            
                let t = val_a / (val_a - val_b);

                let position = [
                    ((x + x0) + (x + x1)) as f32 * 0.5,
                    ((y + y0) + (y + y1)) as f32 * 0.5,
                    ((z + z0) + (z + z1)) as f32 * 0.5,
                ];
            
                let position = (pos_a + (pos_b - pos_a) * t).into();
            
                indices.push(positions.len() as u32);
                edge_to_index.insert(edge_identifier, positions.len() as u32);
                positions.push(position);
            },
        }
    }
}

pub fn marching_cubes_interpolation(
    resolution: usize,
    scalar_field: &ScalarField,
    interpolate: f32,
    normal_weight: f32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>){

    let grid_resolution = resolution + 1;

    let mut grid = Vec::<f32>::with_capacity(grid_resolution * grid_resolution * grid_resolution);

    for z in 0..grid_resolution {
        for y in 0..grid_resolution {
            for x in 0..grid_resolution {
                grid.push(scalar_field(x as f32, y as f32, z as f32));
            }
        }
    }

    let voxel_grid = &move |x, y, z| {
        grid[x + y * grid_resolution + z * grid_resolution * grid_resolution]
    };

    let mut positions = Vec::<[f32; 3]>::new();

    for z in 0..resolution {
        for y in 0..resolution {
            for x in 0..resolution {
                
                let triangulation = get_triangulation(voxel_grid, (x, y, z));

                for edge_index in triangulation {
                    if edge_index == -1 { break; }

                    make_vertex_interpolation(voxel_grid, &mut positions, (x, y, z), edge_index as usize, interpolate);
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
    let grid_resolution = resolution + 1;

    let mut grid = Vec::<f32>::with_capacity(grid_resolution * grid_resolution * grid_resolution);

    for z in 0..grid_resolution {
        for y in 0..grid_resolution {
            for x in 0..grid_resolution {
                grid.push(scalar_field(x as f32, y as f32, z as f32));
            }
        }
    }

    let voxel_grid = &move |x, y, z| {
        grid[x + y * grid_resolution + z * grid_resolution * grid_resolution]
    };

    let mut meshes = Vec::new();


    for z in 0..resolution {
        for y in 0..resolution {
            for x in 0..resolution {
                let mut positions = Vec::<[f32; 3]>::new();
                
                let triangulation = get_triangulation(voxel_grid, (x, y, z));

                for edge_index in triangulation {
                    if edge_index == -1 { break; }

                    make_vertex_interpolation(voxel_grid, &mut positions, (x, y, z), edge_index as usize, 0.0);
                }

                meshes.push(positions);
            }
        }
    }

    meshes
}

fn make_vertex_interpolation(
    voxel_grid: &VoxelGrid,
    positions: &mut Vec<[f32; 3]>,
    (x, y, z): (usize, usize, usize),
    edge_index: usize,
    interpolate: f32,
) {
    let point_index = EDGES[edge_index];

    let (x0, y0, z0) = POINTS[point_index.0];
    let (x1, y1, z1) = POINTS[point_index.1];

    let pos_a: Vec3 = Vec3::new((x + x0) as f32, (y + y0) as f32, (z + z0) as f32);
    let pos_b: Vec3 = Vec3::new((x + x1) as f32, (y + y1) as f32, (z + z1) as f32);

    let val_a = voxel_grid(pos_a.x as usize, pos_a.y as usize, pos_a.z as usize);
    let val_b = voxel_grid(pos_b.x as usize, pos_b.y as usize, pos_b.z as usize);

    let t = val_a / (val_a - val_b);

    let t2 = 0.5 + (t - 0.5) * interpolate;

    let position = (pos_a + (pos_b - pos_a) * t2).into();

    positions.push(position);
}

fn get_triangulation(
    voxel_grid: &VoxelGrid,
    (x, y, z): (usize, usize, usize),
) -> [i8; 15] {
    let mut config_idx = 0b00000000;

    config_idx |= (voxel_grid(  x  ,  y  ,  z  ).is_sign_negative() as u8) << 0;
    config_idx |= (voxel_grid(  x  ,  y  ,z + 1).is_sign_negative() as u8) << 1;
    config_idx |= (voxel_grid(x + 1,  y  ,z + 1).is_sign_negative() as u8) << 2;
    config_idx |= (voxel_grid(x + 1,  y  ,  z  ).is_sign_negative() as u8) << 3;
    config_idx |= (voxel_grid(  x  ,y + 1,  z  ).is_sign_negative() as u8) << 4;
    config_idx |= (voxel_grid(  x  ,y + 1,z + 1).is_sign_negative() as u8) << 5;
    config_idx |= (voxel_grid(x + 1,y + 1,z + 1).is_sign_negative() as u8) << 6;
    config_idx |= (voxel_grid(x + 1,y + 1,  z  ).is_sign_negative() as u8) << 7;
    
    TRIANGULATIONS[config_idx as usize]
}

fn calculate_smooth_normals(
    positions: &Vec<[f32; 3]>,
    indices: &Vec<u32>,
) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0; 3]; positions.len()];

    for idx in 0..indices.len()/3 {
        let i1 = indices[idx * 3 + 0] as usize;
        let i2 = indices[idx * 3 + 1] as usize;
        let i3 = indices[idx * 3 + 2] as usize;

        let p1: Vec3 = positions[i1].into();
        let p2: Vec3 = positions[i2].into();
        let p3: Vec3 = positions[i3].into();

        let n = (p3 - p1).cross(p2 - p1);

        let n1: Vec3 = normals[i1].into();
        let n2: Vec3 = normals[i2].into();
        let n3: Vec3 = normals[i3].into();

        normals[i1] = (n1 + n).into();
        normals[i2] = (n2 + n).into();
        normals[i3] = (n3 + n).into();
    }

    for idx in 0..normals.len() {
        normals[idx] = Into::<Vec3>::into(normals[idx]).normalize().into();
    }

    normals
}

fn gradient(x: f32, y: f32, z: f32, scalar_field: &dyn Fn(f32, f32, f32) -> f32) -> [f32; 3] {
    let e = 1.0;
    let val_x = scalar_field(x + e, y, z) - scalar_field(x - e, y, z);
    let val_y = scalar_field(x, y + e, z) - scalar_field(x, y - e, z);
    let val_z = scalar_field(x, y, z + e) - scalar_field(x, y, z - e);
    return [val_x, val_y, val_z]
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
        
        let n = (p3 - p1).cross(p2 - p1);

        normals[i * 3 + 0] = n.normalize().into();
        normals[i * 3 + 1] = n.normalize().into();
        normals[i * 3 + 2] = n.normalize().into();
    }

    normals
}