#version 100
precision lowp float;

varying vec2 v_Color;

void main() {
    gl_FragColor = vec4(v_Color, 0.0, 1.0);
}