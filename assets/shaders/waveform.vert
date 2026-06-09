// Waveform rendering shader
#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec4 color;
layout(location = 2) in float time_offset;

layout(set = 0, binding = 0) uniform Uniforms {
    mat4 projection;
    float zoom;
    float pan_x;
    float time_scale;
};

layout(location = 0) out vec4 frag_color;
layout(location = 1) out float frag_time;

void main() {
    vec2 transformed = position;
    transformed.x = (position.x - pan_x) * zoom;
    gl_Position = projection * vec4(transformed, 0.0, 1.0);
    frag_color = color;
    frag_time = time_offset;
}