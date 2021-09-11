struct FragmentOutput {
    [[location(0)]] Target0_: vec4<f32>;
};

[[group(0), binding(0)]]
var text_texture: texture_2d<f32>;
[[group(0), binding(1)]]
var image_sampler: sampler;
[[group(0), binding(2)]]
var image_texture: texture_2d<f32>;
var<private> v_uv1: vec2<f32>;
var<private> v_color1: vec4<f32>;
var<private> v_mode1: u32;
var<private> Target0_: vec4<f32>;

fn main1() {
    var text_alpha: f32;
    var image_color: vec4<f32>;

    let _e8: vec2<f32> = v_uv1;
    let _e9: vec4<f32> = textureSample(text_texture, image_sampler, _e8);
    text_alpha = _e9.x;
    let _e13: vec2<f32> = v_uv1;
    let _e14: vec4<f32> = textureSample(image_texture, image_sampler, _e13);
    image_color = _e14;
    let _e16: u32 = v_mode1;
    if ((_e16 == u32(0))) {
        {
            let _e20: vec4<f32> = v_color1;
            let _e24: f32 = text_alpha;
            Target0_ = (_e20 * vec4<f32>(1.0, 1.0, 1.0, _e24));
            return;
        }
    } else {
        let _e27: u32 = v_mode1;
        if ((_e27 == u32(1))) {
            {
                let _e31: vec4<f32> = image_color;
                Target0_ = _e31;
                return;
            }
        } else {
            let _e32: u32 = v_mode1;
            if ((_e32 == u32(2))) {
                {
                    let _e36: vec4<f32> = v_color1;
                    Target0_ = _e36;
                    return;
                }
            } else {
                return;
            }
        }
    }
}

[[stage(fragment)]]
fn main([[location(0)]] v_uv: vec2<f32>, [[location(1)]] v_color: vec4<f32>, [[location(2)]] v_mode: u32) -> FragmentOutput {
    v_uv1 = v_uv;
    v_color1 = v_color;
    v_mode1 = v_mode;
    main1();
    let _e21: vec4<f32> = Target0_;
    return FragmentOutput(_e21);
}
