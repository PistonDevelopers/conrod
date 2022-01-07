// NOTE: This shader requires being manually compiled to SPIR-V in order to
// avoid having downstream users require building shaderc and compiling the
// shader themselves. If you update this shader, be sure to also re-compile it
// and update `vert.spv`. You can do so using `glslangValidator` with the
// following command: `glslangValidator -V -o vert.spv shader.vert`

#version 450
layout(location = 0) in vec2 position;
layout(location = 1) in vec2 tex_coords;
layout(location = 2) in vec4 rgba;
layout(location = 3) in uint mode;
layout(location = 0) out vec2 v_Uv;
layout(location = 1) out vec4 v_Color;
layout(location = 2) flat out uint v_Mode;
void main() {
    v_Uv = tex_coords;
    v_Color = rgba;
    gl_Position = vec4(position, 0.0, 1.0);
    v_Mode = mode;
}