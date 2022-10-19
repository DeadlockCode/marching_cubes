use crate::normal_material::NormalMaterial;

use super::*;

fn cube_to_sphere(v: Vec3) -> Vec3 {
    Vec3::new(
        v.x * (1.0 - v.y * v.y * 0.5 - v.z * v.z * 0.5 + v.y * v.y * v.z * v.z / 3.0).sqrt(),
        v.y * (1.0 - v.z * v.z * 0.5 - v.x * v.x * 0.5 + v.z * v.z * v.x * v.x / 3.0).sqrt(),
        v.z * (1.0 - v.x * v.x * 0.5 - v.y * v.y * 0.5 + v.x * v.x * v.y * v.y / 3.0).sqrt(),
    )
}

fn generate_face(resolution: usize, local_y: Vec3) -> Mesh {
    let local_x = Vec3::new(local_y.y, local_y.z, local_y.x);
    let local_z = local_y.cross(local_x);

    let mut positions = Vec::new();
    positions.resize(resolution * resolution, [0.0, 0.0, 0.0]);
    let mut normals = Vec::new();
    normals.resize(resolution * resolution, [0.0, 0.0, 0.0]);
    let mut indices = Vec::new();
    indices.resize((resolution - 1) * (resolution - 1) * 6, 0 as u32);

    for y in 0..resolution {
        for x in 0..resolution {
            let idx = x + y * resolution;
            let percent = Vec2::new(x as f32, y as f32) / (resolution - 1) as f32;
            let cube = local_y + local_x * (percent.x * 2.0 - 1.0) + local_z * (percent.y * 2.0 - 1.0);
            let sphere = cube_to_sphere(cube);
            positions[idx] = sphere.into();
            normals[idx] = sphere.into();

            if x != resolution - 1 && y != resolution - 1 {
                let idx_2 = (x + y * (resolution - 1)) * 6;
                indices[idx_2 + 0] = (idx                 ) as u32;
                indices[idx_2 + 1] = (idx + resolution + 1) as u32;
                indices[idx_2 + 2] = (idx + resolution    ) as u32;
                indices[idx_2 + 3] = (idx                 ) as u32;
                indices[idx_2 + 4] = (idx              + 1) as u32;
                indices[idx_2 + 5] = (idx + resolution + 1) as u32;
            }
        }
    }

    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    mesh
}

pub fn spawn_cube_sphere(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<NormalMaterial>>,
) {
    let dirs = [
        Vec3::X,
        Vec3::Y,
        Vec3::Z,
        Vec3::NEG_X,
        Vec3::NEG_Y,
        Vec3::NEG_Z,
    ];
    commands.spawn_bundle(SpatialBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    })
    .with_children(|parent| {
        for dir in dirs {
            parent.spawn_bundle(MaterialMeshBundle {
                mesh: meshes.add(generate_face(16, dir)),
                material: materials.add(NormalMaterial{}),
                ..Default::default()
            });
        }
    })
    .insert(Name::new("Planet"));
}