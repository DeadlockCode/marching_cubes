/*use bevy::{prelude::*, pbr::wireframe::WireframeConfig};

fn generate_mesh() -> Mesh {
    todo!();
    Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList)
}

pub fn spawn_dual_contoured_surface(
    mut commands: Commands,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    wireframe_config.global = false;

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(generate_mesh()),
        material: todo!(),
        ..Default::default()
    });
}*/