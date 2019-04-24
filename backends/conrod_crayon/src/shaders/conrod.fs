#version 300 es
precision mediump float;
uniform sampler2D tex;

flat in int v_mode;
in vec2 v_tex_coords;
in vec4 v_color;

out vec4 f_color;

void main() {
    // Text
    
    if (v_mode < int(0.1) ) {
        f_color = v_color * vec4(1.0, 1.0, 1.0, texture(tex, v_tex_coords).r);
        
    // Image
    } else if (v_mode < int(1.1)) {
        f_color = texture(tex, v_tex_coords);
        
    // 2D Geometry
    } else if (v_mode < int(2.1)){
        f_color = v_color;
    }
}