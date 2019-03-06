#version 100
precision lowp float;
varying vec2 v_tex_coords;
varying vec4 v_color;

uniform sampler2D tex;
uniform int mode;
void main() {
    // Text
    if (mode == 0) {
        gl_FragColor = v_color * vec4(1.0, 1.0, 1.0, texture2D(tex, v_tex_coords).r);

    // Image
    } else if (mode == 1) {
        gl_FragColor = texture2D(tex, v_tex_coords);

    // 2D Geometry
    } else if (mode == 2) {
        gl_FragColor = v_color;
    }
}