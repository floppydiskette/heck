#version 330

in vec2 uv;

out vec4 o_colour;

uniform sampler2D mixmap;
uniform sampler2D tex0; // r
uniform sampler2D tex1; // g
uniform sampler2D tex2; // b
uniform sampler2D tex3; // a

uniform float scale = 1;

// uses the mixmap to blend between the 4 textures
// applies scale to uv
void main() {
    // scale the uv
    vec2 scaled_uv = uv * scale;

    vec4 mix = texture(mixmap, uv * scale);
    vec4 r = texture(tex0, uv * scale);
    vec4 g = texture(tex1, uv * scale);
    vec4 b = texture(tex2, uv * scale);
    vec4 a = texture(tex3, uv * scale);
    o_colour = texture(tex0, uv);
}