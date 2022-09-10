#version 330 core

in vec2 uv;
in vec3 normal;
in vec3 frag_pos;

out vec4 o_colour;

uniform sampler2D u_texture;

uniform float u_opacity = 1.0;

vec3 calculate_ambient(float strength, vec3 colour) {
    return strength * colour;
}

void main() {
    vec3 light_pos = vec3(0.0, 0.0, 0.0); // hard coded light position for now
    vec3 light_colour = vec3(1.0, 1.0, 1.0); // hard coded light colour for now

    vec3 norm = normalize(normal);
    vec3 light_dir = normalize(light_pos - frag_pos);

    float diff = max(dot(norm, light_dir), 0.0);
    vec3 diffuse = diff * light_colour;

    vec3 ambient = calculate_ambient(0.1, light_colour);

    vec3 result = (ambient + diffuse) * texture(u_texture, uv).rgb;

    o_colour = vec4(result, u_opacity);
}