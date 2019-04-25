#version 100
precision lowp float;
varying vec2 v_tex_coords;
varying vec4 v_color;
varying float mode;
uniform sampler2D tex;

void main() {
    // Text
    if (mode < 0.1) {
        gl_FragColor = v_color * vec4(1.0, 1.0, 1.0, texture2D(tex, v_tex_coords).r);

    // Image
    } else if (mode < 1.1) {
        gl_FragColor = texture2D(tex, v_tex_coords);

    // 2D Geometry
    } else if (mode < 2.1) {
        gl_FragColor = v_color;
    }
}