#version 330 core

in vec2 uv;
in vec3 normal;
in vec3 frag_pos;

out vec4 o_colour;

uniform vec3 u_camera_pos;

uniform float u_opacity = 1.0;

struct Material {
    sampler2D diffuse;
    sampler2D roughness;
    sampler2D metallic;
    sampler2D normal;
};

uniform Material u_material;

vec3 calculate_ambient(float strength, vec3 colour) {
    return strength * colour;
}

void main() {
    float specular_strength = 0.5;

    vec3 light_pos = vec3(0.0, 0.0, 0.0); // hard coded light position for now
    vec3 light_colour = vec3(1.0, 1.0, 1.0); // hard coded light colour for now

    vec3 norm = normalize(normal);
    vec3 light_dir = normalize(light_pos - frag_pos);

    // diffuse shading

    float diff = max(dot(norm, light_dir), 0.0);
    vec3 diffuse = light_colour * (diff * texture(u_material.diffuse, uv).rgb);

    vec3 view_dir = normalize(u_camera_pos - frag_pos);
    vec3 reflect_dir = reflect(-light_dir, norm);

    // specular shading

    float spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32);
    vec3 specular = light_colour * (spec * texture(u_material.metallic, uv).rgb);

    vec3 ambient = calculate_ambient(0.1, light_colour);

    // roughness
    float roughness = texture(u_material.roughness, uv).r;
    float metallic = texture(u_material.metallic, uv).r;

    vec3 colour = (1.0 - metallic) * diffuse + specular * specular_strength + ambient;
    o_colour = vec4(colour, u_opacity);
}