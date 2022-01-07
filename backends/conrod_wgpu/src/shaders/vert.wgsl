struct VertexOutput {
    [[location(0)]] uv: vec2<f32>;
    [[location(1)]] color: vec4<f32>;
    [[location(2)]] mode: u32;
    [[builtin(position)]] pos: vec4<f32>;
};

[[stage(vertex)]]
fn main(
    [[location(0)]] pos: vec2<f32>,
    [[location(1)]] uv: vec2<f32>,
    [[location(2)]] color: vec4<f32>,
    [[location(3)]] mode: u32,
) -> VertexOutput {
    let out_pos = vec4<f32>(pos * vec2<f32>(1.0, -1.0), 0.0, 1.0);
    return VertexOutput(uv, color, mode, out_pos);
}
