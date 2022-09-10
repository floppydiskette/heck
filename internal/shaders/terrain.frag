#version 330

in vec2 uv;
in vec3 normal;
in vec3 frag_pos;

out vec4 o_colour;

uniform sampler2D mixmap;
uniform sampler2D tex0; // r
uniform sampler2D tex1; // g
uniform sampler2D tex2; // b
uniform sampler2D tex3; // a

uniform float scale = 1;

vec3 calculate_ambient(float strength, vec3 colour) {
    return strength * colour;
}

// uses the mixmap to blend between the 4 textures
// applies scale to uv
void main() {
    // scale the uv
    vec2 scaled_uv = uv * scale;

    vec4 mixmap_tex = texture(mixmap, uv);
    vec3 r = texture2D(tex0, uv * scale).rgb;
    vec3 g = texture2D(tex1, uv * scale).rgb;
    vec3 b = texture2D(tex2, uv * scale).rgb;
    vec3 a = texture2D(tex3, uv * scale).rgb;

    vec3 tex_r = r * mixmap_tex.r;
    vec3 tex_g = mix(r, g, mixmap_tex.g);
    vec3 tex_b = mix(g, b, mixmap_tex.b);
    vec3 tex = mix(b, a, mixmap_tex.a);

    vec3 light_pos = vec3(0.0, 3.0, 0.0); // hard coded light position for now
    vec3 light_colour = vec3(1.0, 1.0, 1.0); // hard coded light colour for now

    vec3 norm = normalize(normal);
    vec3 light_dir = normalize(light_pos - frag_pos);

    float diff = max(dot(norm, light_dir), 0.0);
    vec3 diffuse = diff * light_colour;

    vec3 ambient = calculate_ambient(0.1, light_colour);

    vec3 result = (ambient + diffuse) * tex.rgb;

    o_colour = vec4(result, 1.0);
}