#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec2 Vertex_Uv;
layout(location = 0) out vec2 v_Uv;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};

layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};

layout(set = 2, binding = 0) uniform texture2D Animator_texture;
layout(set = 2, binding = 1) uniform sampler Animator_texture_sampler;

layout(set = 2, binding = 2) uniform Animator_frame {
    uint frame;
};

layout(set = 2, binding = 3) uniform Animator_columns {
    uint columns;
};

layout(set = 2, binding = 4) uniform Animator_rows {
    uint rows;
};

void main() {
    vec2 uv = Vertex_Uv;

    vec2 size = vec2(textureSize(sampler2D(Animator_texture, Animator_texture_sampler), 0));
    vec2 dimensions = size / vec2(float(columns), float(rows));

    float offset_x = float(frame % columns) * dimensions.x;
    float offset_y = float(min(frame / columns, rows - 1)) * dimensions.y;

    vec2 offset = vec2(offset_x, offset_y);

    vec3 position = Vertex_Position * vec3(dimensions, 1.0);
    gl_Position = ViewProj * Model * vec4(position, 1.0);
    v_Uv = (offset / size) + uv / vec2(float(columns), float(rows));
}