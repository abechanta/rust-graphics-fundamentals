@group(0) @binding(0)
var<uniform> angle_y: f32;

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) col: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.pos, 1.0);
    out.clip_position.x *= cos(angle_y);
    out.color = model.col;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
