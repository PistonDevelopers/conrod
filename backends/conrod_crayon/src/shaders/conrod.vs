#version 300 es
precision mediump float;

in vec2 position;
in vec2 tex_coords;
in vec4 color;
in uint mode;

out vec2 v_tex_coords;
out vec4 v_color;
flat out uint v_mode;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    v_tex_coords = tex_coords;
    v_color = color;
    v_mode = mode;
}