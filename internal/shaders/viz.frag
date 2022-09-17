#version 330 core

in vec2 uv;
in vec3 normal;
in vec3 frag_pos;
in vec2 closest_vertex_vector;

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

// point light
struct Light {
    vec3 position;
    vec3 colour;
    float intensity;
};

#define MAX_LIGHTS 100

uniform Light u_lights[MAX_LIGHTS];
uniform int u_light_count;

// makes each side look like a gradient with the outside being darker
vec3 darker_if_closer_to_edge(vec3 colour, vec2 closest_vertex_vector) {
    float closest_vertex_distance = length(closest_vertex_vector);
    float edge_distance = 0.5;
    float edge_fade = 0.5;
    float edge_fade_distance = edge_distance + edge_fade;
    float edge_fade_amount = clamp((edge_fade_distance - closest_vertex_distance) / edge_fade, 0.0, 1.0);
    return colour * edge_fade_amount;
}

void main() {
    vec3 norm = normalize(normal);
    o_colour = vec4(darker_if_closer_to_edge(vec3(0,1,0), closest_vertex_vector), 0.2);
}