use bevy::prelude::*;

use crate::marching_cubes::march_tables;

#[derive(Component)]
pub struct LookAtCamera;
pub struct Timings {
    pub timings: [f32; 5],
    pub delays: [f32; 5],
}

#[derive(Component)]
pub struct MeshHolder;

impl Timings {
    pub fn get_time_in_stage(&self, stage: usize, time: f32) -> f32 {
        if stage > self.timings.len() || stage > self.delays.len() {
            panic!("Stage was either to high or timings and delays isn't filled correctly");
        }

        let mut offset = 0.0;

        for i in 0..(stage + 1) {
            offset += self.delays[i];
            if i != stage {
                offset += self.timings[i];
            }
        }

        return ((time - offset) / self.timings[stage]).clamp(0.0, 1.0);
    }
}

pub fn smoothstep(t: f32) -> f32 {
    if t < 0.0 { return 0.0; }
    if t > 1.0 { return 1.0; }

    return t * t * (3.0 - 2.0 * t);
}

#[derive(Component)]
pub struct Boundary;

pub fn spawn_boundary_cube(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::LineList);
    let mut positions = vec![[0f32; 3]; 24];
    let normals = vec![[0f32; 3]; 24];

    for edge_index in 0..12 {
        let point_index = march_tables::EDGES[edge_index];

        let (x0, y0, z0) = march_tables::POINTS[point_index.0];
        let (x1, y1, z1) = march_tables::POINTS[point_index.1];

        positions[edge_index * 2 + 0] = [x0 as f32 - 0.5, y0 as f32 - 0.5, z0 as f32 - 0.5];
        positions[edge_index * 2 + 1] = [x1 as f32 - 0.5, y1 as f32 - 0.5, z1 as f32 - 0.5];
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(StandardMaterial {
            base_color: Color::DARK_GRAY,
            unlit: true,
            ..Default::default()
        }),
        ..Default::default()
    })
    .insert(Boundary {})
    .insert(Name::new("Bounary"));
}



pub fn look_at_camera_system(
    mut transforms: Query<&mut Transform, (With<LookAtCamera>, Without<Camera3d>)>,
    cameras: Query<&Transform, (With<Camera3d>, Without<LookAtCamera>)>,
) {
    let camera = cameras.single();
    for mut transform in transforms.iter_mut() {
        let rotation = camera.rotation;
        transform.rotation = rotation;
        //transform.rotate_local_y(PI)
    }
}