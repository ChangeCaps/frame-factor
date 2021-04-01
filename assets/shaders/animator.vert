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

void main() {
    vec2 uv = Vertex_Uv;

    vec3 position = Vertex_Position * 20.0;
    gl_Position = ViewProj * Model * vec4(position, 1.0);
    v_Uv = uv;
}