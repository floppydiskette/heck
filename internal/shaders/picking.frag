#version 330

out uvec3 o_colour;

uniform uint u_entity_id;

void main() {
    o_colour = uvec3(u_entity_id, 0, 0);
}