#version 450

layout(location = 0) in vec3 Vertex_Position;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};

layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};

layout(set = 2, binding = 0) uniform ProgressBarMaterial_size {
    vec2 size;
};

void main() {
    vec3 vertex = Vertex_Position * size;
    gl_Position = ViewProj * Model * vec4(vertex, 1.0);
}