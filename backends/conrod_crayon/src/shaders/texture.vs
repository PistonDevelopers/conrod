#version 100
precision lowp float;

attribute vec2 Position;
varying vec2 v_Texcoord;

void main(){
    gl_Position = vec4(Position, 0.0, 1.0);
    v_Texcoord = (Position + vec2(1.0, 1.0)) / 2.0;
}
