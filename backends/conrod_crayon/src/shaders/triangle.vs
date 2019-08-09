#version 100
precision lowp float;

attribute vec2 Position;
varying vec2 v_Color;

void main() {
    gl_Position = vec4(Position, 0.0, 1.0);
    v_Color = vec2(0.0,0.0);
}