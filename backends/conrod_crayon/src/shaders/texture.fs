#version 100
precision lowp float;

varying vec2 v_Texcoord;

uniform sampler2D renderedTexture;

void main() {
    gl_FragColor = vec4(texture2D( renderedTexture, v_Texcoord ).xyz, 1.0);
}