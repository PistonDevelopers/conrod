#version 300 es
precision mediump float;

in vec2 Position;
in vec2 Texcoord0;
in vec4 Color0;
in vec2 Tangent;

out vec2 v_tex_coords;
out vec4 v_color;
flat out uint v_mode;

void main() {
    gl_Position = vec4(Position, 0.0, 1.0);
    v_tex_coords = Texcoord0;
    v_color = Color0;
    v_mode = uint(Tangent.x);
}