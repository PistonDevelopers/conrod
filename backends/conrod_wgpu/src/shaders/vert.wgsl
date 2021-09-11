struct VertexOutput {
    [[location(0)]] v_Uv: vec2<f32>;
    [[location(1)]] v_Color: vec4<f32>;
    [[location(2)]] v_Mode: u32;
    [[builtin(position)]] member: vec4<f32>;
};

var<private> pos1: vec2<f32>;
var<private> uv1: vec2<f32>;
var<private> color1: vec4<f32>;
var<private> mode1: u32;
var<private> v_Uv: vec2<f32>;
var<private> v_Color: vec4<f32>;
var<private> v_Mode: u32;
var<private> gl_Position: vec4<f32>;

fn main1() {
    let _e7: vec2<f32> = uv1;
    v_Uv = _e7;
    let _e8: vec4<f32> = color1;
    v_Color = _e8;
    let _e10: vec2<f32> = pos1;
    gl_Position = vec4<f32>((_e10 * vec2<f32>(1.0, -(1.0))), 0.0, 1.0);
    let _e19: u32 = mode1;
    v_Mode = _e19;
    return;
}

[[stage(vertex)]]
fn main([[location(0)]] pos: vec2<f32>, [[location(1)]] uv: vec2<f32>, [[location(2)]] color: vec4<f32>, [[location(3)]] mode: u32) -> VertexOutput {
    pos1 = pos;
    uv1 = uv;
    color1 = color;
    mode1 = mode;
    main1();
    let _e23: vec2<f32> = v_Uv;
    let _e25: vec4<f32> = v_Color;
    let _e27: u32 = v_Mode;
    let _e29: vec4<f32> = gl_Position;
    return VertexOutput(_e23, _e25, _e27, _e29);
}
