use std::f32::consts::PI;

pub fn circle_fan(
    resolution: u32
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let mut positions = Vec::new();
    let mut indices = Vec::new();

    positions.push([0f32; 3]);
    let step = -2.0 * PI / resolution as f32;
    for i in 0..resolution {
        positions.push([(step * i as f32).cos(), (step * i as f32).sin(), 0.0])
    }

    for i in 0..resolution {
        indices.push(0);
        indices.push(i + 1);
        indices.push((i + 1) % resolution + 1);
    }
    
    
    let normals = vec![[0.0,0.0,1.0]; (resolution + 1) as usize];
    (positions, normals, indices)
}