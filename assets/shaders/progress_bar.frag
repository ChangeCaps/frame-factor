#version 450

layout(location = 0) in vec2 v_Position;
layout(location = 0) out vec4 o_Target;

layout(set = 2, binding = 0) uniform ProgressBarMaterial_size {
    vec2 size;
};

layout(set = 2, binding = 1) uniform ProgressBarMaterial_color_light {
    vec4 color_light;
};

layout(set = 2, binding = 2) uniform ProgressBarMaterial_color_dark {
    vec4 color_dark;
};

layout(set = 2, binding = 3) uniform ProgressBarMaterial_color_bg {
    vec4 color_bg;
};

layout(set = 2, binding = 4) uniform ProgressBarMaterial_color_fg {
    vec4 color_fg;
};

layout(set = 2, binding = 5) buffer ProgressBarMaterial_sections {
    float[] sections;
};

layout(set = 3, binding = 0) uniform ProgressBar_value {
    float value;
};

layout(set = 3, binding = 1) uniform ProgressBar_value_max {
    float value_max;
};

void main() {
    vec4 color = color_light;
    float height = v_Position.y / size.y;
    float width = v_Position.x / size.x;
    float progress = value / value_max;
    float border = 3.0;

    if (height <= 0.05) {
        color = color_dark;
    }

    if (progress < width + 0.5) {
        color = color_bg;
    }

    if (size.x / 2.0 - abs(v_Position.x) < border || size.y / 2.0 - abs(v_Position.y) < border) {
        color = color_fg;
    }

    for (int i = 0; i < sections.length(); i++) {
        if (abs(sections[i] - (width + 0.5)) * size.x < border / 2.0) {
            color = color_fg;
        }
    }

    o_Target = color;
}