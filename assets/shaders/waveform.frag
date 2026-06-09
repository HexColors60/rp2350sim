// Waveform fragment shader
#version 450

layout(location = 0) in vec4 frag_color;
layout(location = 1) in float frag_time;

layout(location = 0) out vec4 out_color;

void main() {
    // Apply slight anti-aliasing based on distance
    out_color = frag_color;
}