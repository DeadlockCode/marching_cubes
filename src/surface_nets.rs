use bevy::utils::HashMap;
use stopwatch::Stopwatch;

type SDF = dyn Fn(f32, f32, f32) -> f32;
type GridSDF = dyn Fn(usize, usize, usize) -> f32;

// Main algorithm driver.
pub fn surface_net(
    resolution: usize,
    signed_distance_field: &SDF,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let sw = Stopwatch::start_new();

    let axis_length = resolution + 1;
    let grid = coords(axis_length)
        .map(|(x, y, z)| signed_distance_field(x as f32, y as f32, z as f32))
        .collect::<Vec<_>>();
    let discrete_scalar_field = &move |x, y, z| {
        unsafe { *grid.get_unchecked(x + y * axis_length + z * axis_length * axis_length) }
    };

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut grid_to_index = HashMap::new();
    // Find all vertex positions. Addtionally, create a hashmap from grid
    // position to index.
    for coord in coords(resolution) {
        if let Some((center, normal)) = find_center(discrete_scalar_field, coord) {
            grid_to_index.insert(coord, positions.len());
            positions.push(center);
            normals.push(normal);
        }
    }
    // Find all triangles, in the form of [index, index, index] triples.
    let mut indices = Vec::new();
    make_all_triangles(
        discrete_scalar_field,
        resolution,
        &grid_to_index,
        &positions,
        &mut indices,
    );
    println!("Surface nets took: {}ms", sw.elapsed_ms());
    (positions, normals, indices)
}

// Iterator over all integer points in a 3d cube from 0 to size
fn coords(size: usize) -> impl Iterator<Item = (usize, usize, usize)> {
    (0..size)
        .flat_map(move |x| (0..size).map(move |y| (x, y)))
        .flat_map(move |(x, y)| (0..size).map(move |z| (x, y, z)))
}

// List of all edges in a cube.
const OFFSETS: [(usize, usize); 12] = [
    (0b000, 0b001), // ((0, 0, 0), (0, 0, 1)),
    (0b000, 0b010), // ((0, 0, 0), (0, 1, 0)),
    (0b000, 0b100), // ((0, 0, 0), (1, 0, 0)),
    (0b001, 0b011), // ((0, 0, 1), (0, 1, 1)),
    (0b001, 0b101), // ((0, 0, 1), (1, 0, 1)),
    (0b010, 0b011), // ((0, 1, 0), (0, 1, 1)),
    (0b010, 0b110), // ((0, 1, 0), (1, 1, 0)),
    (0b011, 0b111), // ((0, 1, 1), (1, 1, 1)),
    (0b100, 0b101), // ((1, 0, 0), (1, 0, 1)),
    (0b100, 0b110), // ((1, 0, 0), (1, 1, 0)),
    (0b101, 0b111), // ((1, 0, 1), (1, 1, 1)),
    (0b110, 0b111), // ((1, 1, 0), (1, 1, 1)),
];

// Find the vertex position for this grid: it will be somewhere within the cube
// with coordinates [0,1].
// How? First, for each edge in the cube, find if that edge crosses the SDF
// boundary - i.e. one point is positive, one point is negative.
// Second, calculate the "weighted midpoint" between these points (see
// find_edge).
// Third, take the average of all these points for all edges (for edges that
// have crossings).
// There are more complicated and better algorithms than this, but this is
// simple and easy to implement.
// Returns: (pos, normal)
fn find_center(
    discrete_scalar_field: &GridSDF,
    coord: (usize, usize, usize),
) -> Option<([f32; 3], [f32; 3])> {
    let mut values = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    for (x, value) in values.iter_mut().enumerate() {
        *value = discrete_scalar_field(
            coord.0 + (x & 1),
            coord.1 + ((x >> 1) & 1),
            coord.2 + ((x >> 2) & 1),
        );
    }
    let edges = OFFSETS.iter().filter_map(|&(offset1, offset2)| {
        find_edge(offset1, offset2, values[offset1], values[offset2])
    });
    let mut count = 0;
    let mut sum = [0.0, 0.0, 0.0];
    for edge in edges {
        count += 1;
        sum[0] += edge[0];
        sum[1] += edge[1];
        sum[2] += edge[2];
    }
    if count == 0 {
        None
    } else {
        let normal_x = (values[0b001] + values[0b011] + values[0b101] + values[0b111])
            - (values[0b000] + values[0b010] + values[0b100] + values[0b110]);
        let normal_y = (values[0b010] + values[0b011] + values[0b110] + values[0b111])
            - (values[0b000] + values[0b001] + values[0b100] + values[0b101]);
        let normal_z = (values[0b100] + values[0b101] + values[0b110] + values[0b111])
            - (values[0b000] + values[0b001] + values[0b010] + values[0b011]);
        let normal_len = (normal_x * normal_x + normal_y * normal_y + normal_z * normal_z).sqrt();
        Some((
            [
                sum[0] / count as f32 + coord.0 as f32,
                sum[1] / count as f32 + coord.1 as f32,
                sum[2] / count as f32 + coord.2 as f32,
            ],
            [
                normal_x / normal_len,
                normal_y / normal_len,
                normal_z / normal_len,
            ],
        ))
    }
}

// Given two points, A and B, find the point between them where the SDF is zero.
// (This might not exist).
// A and B are specified via A=coord+offset1 and B=coord+offset2, because code
// is weird.
fn find_edge(offset1: usize, offset2: usize, value1: f32, value2: f32) -> Option<[f32; 3]> {
    if (value1 < 0.0) == (value2 < 0.0) {
        return None;
    }
    let interp = value1 / (value1 - value2);
    let point = [
        (offset1 & 1) as f32 * (1.0 - interp) + (offset2 & 1) as f32 * interp,
        ((offset1 >> 1) & 1) as f32 * (1.0 - interp) + ((offset2 >> 1) & 1) as f32 * interp,
        ((offset1 >> 2) & 1) as f32 * (1.0 - interp) + ((offset2 >> 2) & 1) as f32 * interp,
    ];
    Some(point)
}

// For every edge that crosses the boundary, make a quad between the
// "centers" of the four cubes touching that boundary. (Well, really, two
// triangles) The "centers" are actually the vertex positions, found earlier.
// (Also, make sure the triangles are facing the right way)
// There's some hellish off-by-one conditions and whatnot that make this code
// really gross.
fn make_all_triangles(
    discrete_scalar_field: &GridSDF,
    resolution: usize,
    grid_to_index: &HashMap<(usize, usize, usize), usize>,
    positions: &[[f32; 3]],
    indices: &mut Vec<u32>,
) {
    for coord in coords(resolution) {
        // TODO: Cache discrete_scalar_field(coord), it's called three times here.
        // Do edges parallel with the X axis
        if coord.1 != 0 && coord.2 != 0 {
            make_triangle(
                discrete_scalar_field,
                grid_to_index,
                positions,
                indices,
                coord,
                (1, 0, 0),
                (0, 1, 0),
                (0, 0, 1),
            );
        }
        // Do edges parallel with the Y axis
        if coord.0 != 0 && coord.2 != 0 {
            make_triangle(
                discrete_scalar_field,
                grid_to_index,
                positions,
                indices,
                coord,
                (0, 1, 0),
                (0, 0, 1),
                (1, 0, 0),
            );
        }
        // Do edges parallel with the Z axis
        if coord.0 != 0 && coord.1 != 0 {
            make_triangle(
                discrete_scalar_field,
                grid_to_index,
                positions,
                indices,
                coord,
                (0, 0, 1),
                (1, 0, 0),
                (0, 1, 0),
            );
        }
    }
}

fn make_triangle(
    discrete_scalar_field: &GridSDF,
    grid_to_index: &HashMap<(usize, usize, usize), usize>,
    positions: &[[f32; 3]],
    indices: &mut Vec<u32>,
    coord: (usize, usize, usize),
    offset: (usize, usize, usize),
    axis1: (usize, usize, usize),
    axis2: (usize, usize, usize),
) {
    let face_result = is_face(discrete_scalar_field, coord, offset);
    if let FaceResult::NoFace = face_result {
        return;
    }
    // The triangle points, viewed face-front, look like this:
    // v1 v3
    // v2 v4
    let v1 = *grid_to_index.get(&(coord.0, coord.1, coord.2)).unwrap();
    let v2 = *grid_to_index
        .get(&(coord.0 - axis1.0, coord.1 - axis1.1, coord.2 - axis1.2))
        .unwrap();
    let v3 = *grid_to_index
        .get(&(coord.0 - axis2.0, coord.1 - axis2.1, coord.2 - axis2.2))
        .unwrap();
    let v4 = *grid_to_index
        .get(&(
            coord.0 - axis1.0 - axis2.0,
            coord.1 - axis1.1 - axis2.1,
            coord.2 - axis1.2 - axis2.2,
        )).unwrap();
    // optional addition to algorithm: split quad to triangles in a certain way
    let p1 = positions[v1];
    let p2 = positions[v2];
    let p3 = positions[v3];
    let p4 = positions[v4];
    fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
        let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
        d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
    }
    let d14 = dist(p1, p4);
    let d23 = dist(p2, p3);
    // Split the quad along the shorter axis, rather than the longer one.
    if d14 < d23 {
        match face_result {
            FaceResult::NoFace => (),
            FaceResult::FacePositive => {
                indices.push(v1 as u32);
                indices.push(v2 as u32);
                indices.push(v4 as u32);

                indices.push(v1 as u32);
                indices.push(v4 as u32);
                indices.push(v3 as u32);
            }
            FaceResult::FaceNegative => {
                indices.push(v1 as u32);
                indices.push(v4 as u32);
                indices.push(v2 as u32);

                indices.push(v1 as u32);
                indices.push(v3 as u32);
                indices.push(v4 as u32);
            }
        }
    } else {
        match face_result {
            FaceResult::NoFace => (),
            FaceResult::FacePositive => {
                indices.push(v2 as u32);
                indices.push(v4 as u32);
                indices.push(v3 as u32);

                indices.push(v2 as u32);
                indices.push(v3 as u32);
                indices.push(v1 as u32);
            }
            FaceResult::FaceNegative => {
                indices.push(v2 as u32);
                indices.push(v3 as u32);
                indices.push(v4 as u32);

                indices.push(v2 as u32);
                indices.push(v1 as u32);
                indices.push(v3 as u32);
            }
        }
    }
}

enum FaceResult {
    NoFace,
    FacePositive,
    FaceNegative,
}

// Determine if the sign of the SDF flips between coord and (coord+offset)
fn is_face(
    discrete_scalar_field: &GridSDF,
    coord: (usize, usize, usize),
    offset: (usize, usize, usize),
) -> FaceResult {
    let other = (coord.0 + offset.0, coord.1 + offset.1, coord.2 + offset.2);
    match (
        discrete_scalar_field(coord.0, coord.1, coord.2) < 0.0,
        discrete_scalar_field(other.0, other.1, other.2) < 0.0,
    ) {
        (true, false) => FaceResult::FacePositive,
        (false, true) => FaceResult::FaceNegative,
        _ => FaceResult::NoFace,
    }
}