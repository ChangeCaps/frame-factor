#version 450

layout(location = 0) out vec4 o_Target;
layout(location = 0) in vec2 v_Uv;

layout(set = 2, binding = 0) uniform texture2D Animator_texture;
layout(set = 2, binding = 1) uniform sampler Animator_texture_sampler;

void main() {
    vec4 color = texture(sampler2D(Animator_texture, Animator_texture_sampler), v_Uv);

    if (color.a < 0.01) {
        discard;
    }

    o_Target = color;
}