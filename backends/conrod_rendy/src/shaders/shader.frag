// NOTE: This shader requires being manually compiled to SPIR-V in order to
// avoid having downstream users require building shaderc and compiling the
// shader themselves. If you update this shader, be sure to also re-compile it
// and update `frag.spv`. You can do so using `glslangValidator` with the
// following command: `glslangValidator -V -o frag.spv shader.frag`

#version 450

layout(set = 0, binding = 0) uniform sampler2D t_TextColor;
layout(set = 0, binding = 1) uniform sampler2D t_ImgColor;

layout(location = 0) in vec2 v_Uv;
layout(location = 1) in vec4 v_Color;
layout(location = 2) flat in uint v_Mode;

layout(location = 0) out vec4 Target0;

void main() {
    // Text
    if (v_Mode == uint(0)) {
        Target0 = v_Color * vec4(1.0, 1.0, 1.0, texture(t_TextColor, v_Uv).r);

    // Image
    } else if (v_Mode == uint(1)) {
        Target0 = texture(t_ImgColor, v_Uv);

    // 2D Geometry
    } else if (v_Mode == uint(2)) {
        Target0 = v_Color;
    }
}
