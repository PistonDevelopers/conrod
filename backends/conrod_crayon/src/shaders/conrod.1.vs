#version 100
precision lowp float;

attribute lowp vec2 Position;
attribute lowp vec2 Texcoord0;
attribute lowp vec4 Color0;
attribute lowp float Weight;

varying vec2 v_tex_coords;
varying vec4 v_color;
varying float mode;
void main() {
    gl_Position = vec4(Position, 0.0, 1.0);
    v_tex_coords = Texcoord0;
    v_color = Color0;
    mode = Weight;
}