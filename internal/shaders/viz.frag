#version 330 core

layout (location = 5) out ivec4 fc;

in vec2 uv;
in mat3 TBN;
in vec3 frag_pos;

uniform sampler2D diffuse;
uniform sampler2D specular;
uniform sampler2D normalmap;

uniform float opacity = 1.0;
uniform bool unlit = false;

uniform int type;

void main() {
    fc = ivec4(type, 1, 1, 1);
}