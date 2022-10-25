struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
};

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    var output_color = vec4<f32>(1.0 - (input.world_normal + 1.0) / 2.0, 1.0);
    var f = 1.0 / exp2(input.clip_position.z * 1000.0);
    return mix(output_color, vec4<f32>(0.37, 1.0, 0.73, 1.0), f);
}