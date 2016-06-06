#version 400

uniform sampler2D circle_texture;
// uniform sampler2D depth_texture;

in vec2 g_uv;
// in float g_depth;

out vec4 f_color;

vec3 color = vec3(0.17968, 0.79687, 0.44140); // green

void    main() {
    f_color = texture(circle_texture, g_uv) * vec4(color, 1.0);
    // http://wallaceyuen.com/?p=62
    if (f_color.a < 1.0) {
        discard;
    }
}
