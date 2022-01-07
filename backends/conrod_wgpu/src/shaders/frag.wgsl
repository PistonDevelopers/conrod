struct FragmentOutput {
    [[location(0)]] color: vec4<f32>;
};

[[group(0), binding(0)]]
var text_texture: texture_2d<f32>;
[[group(0), binding(1)]]
var image_sampler: sampler;
[[group(0), binding(2)]]
var image_texture: texture_2d<f32>;

[[stage(fragment)]]
fn main(
    [[location(0)]] uv: vec2<f32>,
    [[location(1)]] color: vec4<f32>,
    [[location(2)]] mode: u32,
) -> FragmentOutput {
    var text_color: vec4<f32> = textureSample(text_texture, image_sampler, uv);
    var image_color: vec4<f32> = textureSample(image_texture, image_sampler, uv);
    var text_alpha: f32 = text_color.x;
    var out_color: vec4<f32> = vec4<f32>(0.5, 0.0, 0.0, 1.0);
    if (mode == u32(0)) {
        out_color = color * vec4<f32>(1.0, 1.0, 1.0, text_alpha);
    } else {
        if (mode == u32(1)) {
            out_color = image_color;
        } else {
            if (mode == u32(2)) {
                out_color = color;
            }
        }
    }
    return FragmentOutput(out_color);
}
