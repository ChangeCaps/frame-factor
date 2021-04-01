#version 450

layout(location = 0) out vec4 o_Target;
layout(location = 0) in vec2 v_Uv;

#ifdef ANIMATOR_TEXTURE
layout(set = 2, binding = 0) uniform texture2D Animator_texture;
layout(set = 2, binding = 1) uniform sampler Animator_texture_sampler;
#endif

layout(set = 2, binding = 2) uniform Animator_frame {
    uint frame;
};

void main() {
#ifdef ANIMATOR_TEXTURE
    o_Target = texture(sampler2D(Animator_texture, Animator_texture_sampler), v_Uv);
#endif
}