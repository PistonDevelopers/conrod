#version 100
precision lowp float;

varying vec2 v_Texcoord;

uniform sampler2D renderedTexture;
uniform float time;

void main() {
    vec2 offset = 0.025*vec2(sin(time+1024.0*v_Texcoord.x), cos(time+768.0*v_Texcoord.y));
    gl_FragColor = vec4(texture2D( renderedTexture, v_Texcoord + offset ).xyz, 1.0);
}