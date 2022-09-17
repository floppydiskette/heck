#version 330 core

layout(location = 0) in vec3 in_pos;
layout(location = 1) in vec2 in_uv;
layout(location = 2) in vec3 in_normal;

out vec2 uv;
out vec3 normal;
out vec3 frag_pos;
out vec2 closest_vertex_vector;

uniform mat4 u_mvp;
uniform mat4 u_model;

vec2 calculate_closest_vertex_vector(vec3 pos) {
    vec3 closest_vertex = vec3(0.0, 0.0, 0.0);
    float closest_distance = 1000000.0;
    for (int i = 0; i < 8; i++) {
        vec3 vertex = vec3(0.0, 0.0, 0.0);
        if (i == 0) {
            vertex = vec3(-0.5, -0.5, -0.5);
        } else if (i == 1) {
            vertex = vec3(-0.5, -0.5, 0.5);
        } else if (i == 2) {
            vertex = vec3(-0.5, 0.5, -0.5);
        } else if (i == 3) {
            vertex = vec3(-0.5, 0.5, 0.5);
        } else if (i == 4) {
            vertex = vec3(0.5, -0.5, -0.5);
        } else if (i == 5) {
            vertex = vec3(0.5, -0.5, 0.5);
        } else if (i == 6) {
            vertex = vec3(0.5, 0.5, -0.5);
        } else if (i == 7) {
            vertex = vec3(0.5, 0.5, 0.5);
        }
        float distance = length(pos - vertex);
        if (distance < closest_distance) {
            closest_distance = distance;
            closest_vertex = vertex;
        }
    }
    return closest_vertex.xy;
}

void main()
{
    gl_Position = u_mvp * vec4(in_pos, 1.0);
    frag_pos = vec3(u_model * vec4(in_pos, 1.0));
    closest_vertex_vector = calculate_closest_vertex_vector(frag_pos);

    uv = in_uv;
    normal = in_normal;
}