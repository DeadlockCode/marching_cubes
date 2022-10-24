use super::*;

use crate::{visualization_helper::*, marching_cubes::march_tables, normal_material::NormalMaterial};

use bevy::{render::{mesh::Indices, render_resource::Face}, log::LogSettings};
use bevy_inspector_egui::{WorldInspectorPlugin, RegisterInspectable};

use ttf2mesh::{Value, TTFFile};

enum TimeStage {
    CubeExpantion,
    PointRevealing,
    NumberReplacement,
    EdgeReplacement,
    MeshBuilding,
}

const TIMINGS: Timings = Timings {
    timings:  [1.0, 4.0, 4.0, 6.0, 1.0],
    delays: [2.0, 1.0, 1.0, 1.0, 1.0],
};


#[derive(Component)]
struct GridPoint {
    index: usize,
}

#[derive(Component)]
struct EdgeNumber {
    index: usize,
}

#[derive(Component)]
struct CornerNumber {
    index: usize,
    active: bool,
}

fn scalar_field(i: f32, j: f32, k: f32) -> f32 {
    let mul = (128.0/17.0);

    let (x, y, z) = (i * mul - 4.0, j * mul - 4.0, k * mul - 4.0);

    (x-2.0)*(x-2.0)*(x+2.0)*(x+2.0) + (y-2.0)*(y-2.0)*(y+2.0)*(y+2.0) + (z-2.0)*(z-2.0)*(z+2.0)*(z+2.0) + 3.0*(x*x*y*y+x*x*z*z+y*y*z*z) + 6.0*x*y*z - 10.0*(x*x+y*y+z*z) + 22.0
}
fn sphere(i: f32, j: f32, k: f32) -> f32 {
    let (x, y, z) = (i - 0.5, j - 0.5, k - 0.5);

    x*x + y*y + z*z - 0.2
}

const SCALAR_FIELD: &dyn Fn(f32, f32, f32) -> f32 = &sphere;

pub fn start() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb_u8(20, 20, 20)))
        .insert_resource(WindowDescriptor {
            width: WIDTH,
            height: HEIGHT,
            title: "Marching Cubes".to_string(),
            resizable: true,
            ..Default::default()
        })
        .insert_resource(LogSettings {
            level: bevy::log::Level::WARN,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())

        .add_startup_system(spawn_camera)
        .add_system(camera_system)


        .add_startup_system(spawn_point_light)

        .add_startup_system(spawn_boundary_cube)
        .add_system(cube_expantion_system)

        .add_startup_system(spawn_points)
        .add_system(point_system)

        .add_startup_system(spawn_numbers)
        .add_system(corner_number_system)
        .add_system(edge_number_system)

        .add_system(activate_corner_system)

        .add_startup_system(spawn_mesh_holder)
        .add_system(mesh_system)

        .add_system(look_at_camera_system)

        .run();
}

fn cube_expantion_system(
    mut boundaries: Query<&mut Transform, With<Boundary>>,
    time: Res<Time>,
) {
    let t = TIMINGS.get_time_in_stage(TimeStage::CubeExpantion as usize, time.seconds_since_startup() as f32);

    let mut boundary = boundaries.single_mut();

    boundary.scale = Vec3::splat(t * (2.0 - t));
}

fn spawn_points(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (positions, normals, indices) = circle_fan::circle_fan(16);

    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    let shared_mesh = meshes.add(mesh);
    let shared_material = materials.add(StandardMaterial {
        base_color: Color::WHITE, 
        unlit: true,
        ..Default::default()
    });

    commands.spawn_bundle(
        SpatialBundle::default()
    ).insert(Name::new("Points"))
    .with_children(|parent| {
        for i in 0..8usize {
            let p = march_tables::POINTS[i];

            parent.spawn_bundle(MaterialMeshBundle {
                mesh: shared_mesh.clone(),
                material: shared_material.clone(),
                transform: Transform::from_translation(Vec3::new(p.0 as f32, p.1 as f32, p.2 as f32) - Vec3::splat(0.5)).with_scale(Vec3::splat(0.025)),
                ..Default::default()
            }).insert(GridPoint { index: i })
            .insert(LookAtCamera)
            .insert(Name::new(i.to_string()));
        }
    });
}

fn point_system(
    mut points: Query<(&mut Transform, &mut Visibility, &GridPoint)>,
    time: Res<Time>,
) {
    let t0 = TIMINGS.get_time_in_stage(TimeStage::CubeExpantion as usize, time.seconds_since_startup() as f32);
    let t1 = TIMINGS.get_time_in_stage(TimeStage::PointRevealing as usize, time.seconds_since_startup() as f32);
    let t2 = TIMINGS.get_time_in_stage(TimeStage::NumberReplacement as usize, time.seconds_since_startup() as f32);

    let current_point = (t1 * 8.0) as usize;
    let current_number = (t2 * 8.0) as usize;
    for (mut transform, mut visibility, point) in points.iter_mut() {
        let p = march_tables::POINTS[point.index];
        transform.translation = (Vec3::new(p.0 as f32, p.1 as f32, p.2 as f32) - Vec3::splat(0.5)) * (t0 * (2.0 - t0));
        visibility.is_visible = current_point >= point.index && current_number <= point.index;
    }
}

fn spawn_numbers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut font = ttf2mesh::TTFFile::from_file("C:\\Projects\\marching_cubes\\assets\\RobotoMono-Regular.ttf").unwrap();

    let shared_material = materials.add(StandardMaterial {
        base_color: Color::DARK_GRAY,
        unlit: true,
        ..Default::default()
    });

    commands.spawn_bundle(SpatialBundle::default())
    .insert(Name::new("Corners"))
    .with_children(|builder| {
        for i in 0..8usize {

            let num = ('0' as u8 + i as u8) as char;

            let (positions, normals, indices) = mesh_from_char(&mut font, num);
            
            let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
            mesh.set_indices(Some(Indices::U32(indices)));
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

            let p = march_tables::POINTS[i];

            builder.spawn_bundle(PbrBundle {
                mesh: meshes.add(mesh),
                transform: Transform::from_translation(Vec3::new(p.0 as f32, p.1 as f32, p.2 as f32) - Vec3::splat(0.5)).with_scale(Vec3::splat(0.1)),
                material: materials.add(StandardMaterial {
                    base_color: Color::DARK_GRAY,
                    unlit: true,
                    ..Default::default()
                }),
                ..Default::default()
            })
            .insert(CornerNumber { index: i, active: false })
            .insert(LookAtCamera)
            .insert(Name::new(i.to_string()));
        }
    });


    commands.spawn_bundle(SpatialBundle::default())
    .insert(Name::new("Edges"))
    .with_children(|builder| {
        for i in 0..12usize {
            if i < 10 {
                let num = ('0' as u8 + i as u8) as char;
    
                let (positions, normals, indices) = mesh_from_char(&mut font, num);
            
                let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
                mesh.set_indices(Some(Indices::U32(indices)));
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

                let edge = march_tables::EDGES[i];

                let p1 = march_tables::POINTS[edge.0];
                let p2 = march_tables::POINTS[edge.1];
    
                let p = ((p1.0 + p2.0) as f32 * 0.5, (p1.1 + p2.1) as f32 * 0.5, (p1.2 + p2.2) as f32 * 0.5);
    
                builder.spawn_bundle(PbrBundle {
                    mesh: meshes.add(mesh),
                    transform: Transform::from_translation(Vec3::new(p.0 as f32, p.1 as f32, p.2 as f32) - Vec3::splat(0.5)).with_scale(Vec3::splat(0.1)),
                    material: shared_material.clone(),
                    ..Default::default()
                })
                .insert(EdgeNumber { index: i })
                .insert(LookAtCamera)
                .insert(Name::new(i.to_string()));
            }
            else {
                let num = ('0' as u8 + (i & 1) as u8) as char;

                let (positions0, normals0, indices0) = mesh_from_char(&mut font, '1');
                let (positions1, normals1, indices1) = mesh_from_char(&mut font, num);

                let mut mesh0 = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
                mesh0.set_indices(Some(Indices::U32(indices0)));
                mesh0.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions0);
                mesh0.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals0);

                let mut mesh1 = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
                mesh1.set_indices(Some(Indices::U32(indices1)));
                mesh1.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions1);
                mesh1.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals1);

                let edge = march_tables::EDGES[i];

                let p1 = march_tables::POINTS[edge.0];
                let p2 = march_tables::POINTS[edge.1];
    
                let p = ((p1.0 + p2.0) as f32 * 0.5, (p1.1 + p2.1) as f32 * 0.5, (p1.2 + p2.2) as f32 * 0.5);

                builder.spawn_bundle(SpatialBundle {
                    transform : Transform::from_translation(Vec3::new(p.0 as f32, p.1 as f32, p.2 as f32) - Vec3::splat(0.5)).with_scale(Vec3::splat(0.1)),
                    ..Default::default()
                })
                .insert(EdgeNumber { index: i })
                .insert(LookAtCamera)
                .insert(Name::new(i.to_string()))
                .with_children(|builder| {
                    builder.spawn_bundle(PbrBundle {
                        mesh: meshes.add(mesh0),
                        transform: Transform::from_xyz(0.25, 0.0, 0.0),
                        material: shared_material.clone(),
                        ..Default::default()
                    });

                    builder.spawn_bundle(PbrBundle {
                        mesh: meshes.add(mesh1),
                        transform: Transform::from_xyz(-0.25, 0.0, 0.0),
                        material: shared_material.clone(),
                        ..Default::default()
                    });
                });
            }
        }
    });
}

fn corner_number_system(
    mut numbers: Query<(&mut Visibility, &CornerNumber)>,
    time: Res<Time>,
) {
    let t = TIMINGS.get_time_in_stage(TimeStage::NumberReplacement as usize, time.seconds_since_startup() as f32);

    let current = (t * 8.0) as usize;
    for (mut visibility, number) in numbers.iter_mut() {
        visibility.is_visible = current > number.index;
    }
}

fn edge_number_system(
    mut numbers: Query<(&mut Visibility, &EdgeNumber)>,
    time: Res<Time>,
) {
    let t = TIMINGS.get_time_in_stage(TimeStage::EdgeReplacement as usize, time.seconds_since_startup() as f32);

    let current = (t * 12.0) as usize;
    for (mut visibility, number) in numbers.iter_mut() {
        visibility.is_visible = current > number.index;
    }
}

fn activate_corner_system(
    mut numbers: Query<(&Transform, &Handle<StandardMaterial>, &mut CornerNumber)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    windows: Res<Windows>,
    buttons: Res<Input<MouseButton>>,
    q_camera: Query<(&Camera, &Transform)>,
) {
    let (camera, camera_transform) = q_camera.single();
    let window = windows.get_primary().unwrap();

    if buttons.just_pressed(MouseButton::Left) {
        if let Some(screen_pos) = window.cursor_position() {
            let window_size = Vec2::new(window.width() as f32, window.height() as f32);

            let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

            let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

            let world_pos = camera_transform.translation;
            let world_dir = (ndc_to_world.project_point3(ndc.extend(1.0)) - world_pos).normalize();

            for (transform, material_handle, mut corner) in numbers.iter_mut() {
                let dir_to_number = (transform.translation - world_pos).normalize();

                let accuracy = world_dir.dot(dir_to_number);
                if accuracy > 0.999 {
                    corner.active = !corner.active;
                }

                let mut material = materials.get_mut(material_handle).unwrap();
                if corner.active {
                    material.base_color = Color::WHITE;
                }
                else {
                    material.base_color = Color::DARK_GRAY;
                }
            }

            let string: String = numbers.iter().map(|(_, _, corner)| {
                ('0' as u8 + corner.active as u8) as char
            }).collect();
            println!("{}", string);
        }
    }
}

fn spawn_mesh_holder(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn_bundle(MaterialMeshBundle {
        mesh: meshes.add(Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList)),
        material: materials.add(StandardMaterial {
            base_color: Color::ORANGE_RED,
            emissive: Color::BLACK,
            perceptual_roughness: 1.0,
            metallic: 0.0,
            reflectance: 0.0,
            cull_mode: None,
            ..Default::default()
        }),
        transform: Transform::from_translation(-Vec3::splat(0.5)),
        ..Default::default()
    }).insert(Name::new("Mesh"))
    .insert(MeshHolder {});
}

fn mesh_system(
    q_mesh: Query<&Handle<Mesh>, (With<MeshHolder>, Without<CornerNumber>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    numbers: Query<(&Handle<StandardMaterial>, &CornerNumber)>,
) {
    let mut triangulation_index = 0;

    for (index, (_, number)) in numbers.iter().enumerate() {
        if number.active { triangulation_index |= 1 << index; };
    }

    let triangulation = march_tables::TRIANGULATIONS[triangulation_index];

    let mut positions = Vec::<[f32; 3]>::new();

    for edge_index in triangulation {
        if edge_index == -1 { break; }

        let point_index = march_tables::EDGES[edge_index as usize];

        let (x0, y0, z0) = march_tables::POINTS[point_index.0];
        let (x1, y1, z1) = march_tables::POINTS[point_index.1];

        let pos_a: Vec3 = Vec3::new(x0 as f32, y0 as f32, z0 as f32);
        let pos_b: Vec3 = Vec3::new(x1 as f32, y1 as f32, z1 as f32);
    
        let position = ((pos_a + pos_b) * 0.5).into();

        positions.push(position);
    }

    let mut normals = Vec::<[f32; 3]>::new();

    for index in 0..positions.len()/3 {
        let p1: Vec3 = positions[index * 3 + 0].into();
        let p2: Vec3 = positions[index * 3 + 1].into();
        let p3: Vec3 = positions[index * 3 + 2].into();

        let n = (p2 - p1).cross(p3 - p1).into();

        normals.push(n);
        normals.push(n);
        normals.push(n);
    }

    let mesh = meshes.get_mut(q_mesh.single()).unwrap();

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
}

fn mesh_from_char(
    font: &mut TTFFile,
    char: char,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let mut glyph = font.glyph_from_char(char).unwrap();

    let glyph_mesh = glyph.to_2d_mesh(ttf2mesh::Quality::High).unwrap();

    let positions = glyph_mesh.iter_vertices()
        .map(|v| {
            let v = v.val();
            [-(v.0 - 0.3), (v.1 - 0.36), 0.0]
        })
        .collect::<Vec<_>>();

    let mut indices = Vec::<u32>::new();

    glyph_mesh.iter_faces()
        .for_each(|f| {
            let f = f.val();
            indices.push(f.0 as u32);
            indices.push(f.1 as u32);
            indices.push(f.2 as u32);
        });

    let normals = vec![[0.0, 0.0, -1.0]; positions.len()];

    (positions, normals, indices)
}
