// NOTE: This shader requires being manually compiled to SPIR-V in order to
// avoid having downstream users require building shaderc and compiling the
// shader themselves. If you update this shader, be sure to also re-compile it
// and update `frag.spv`. You can do so using `glslangValidator` with the
// following command: `glslangValidator -V -o frag.spv shader.frag`

#version 450

layout(set = 0, binding = 0) uniform texture2D text_texture;
layout(set = 0, binding = 1) uniform sampler image_sampler;
layout(set = 0, binding = 2) uniform texture2D image_texture;

layout(location = 0) in vec2 v_uv;
layout(location = 1) in vec4 v_color;
layout(location = 2) flat in uint v_mode;

layout(location = 0) out vec4 Target0;

void main() {
    // Text
    if (v_mode == uint(0)) {
        float a = texture(sampler2D(text_texture, image_sampler), v_uv).r;
        Target0 = v_color * vec4(1.0, 1.0, 1.0, a);

    // Image
    } else if (v_mode == uint(1)) {
        Target0 = texture(sampler2D(image_texture, image_sampler), v_uv);

    // 2D Geometry
    } else if (v_mode == uint(2)) {
        Target0 = v_color;
    }
}
