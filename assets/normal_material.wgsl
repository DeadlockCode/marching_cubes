struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
};

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    var output_color = vec4<f32>(1.0 - (input.world_normal + 1.0) / 2.0, 1.0);
    var s = -input.clip_position.z * 100.0;
    var t = exp(s * s); // fix this bullshit
    return output_color + vec4(1.0, 1.0, 1.0, 1.0) * s;
}