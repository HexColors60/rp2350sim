// Memory heatmap shader
#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 tex_coord;

layout(set = 0, binding = 0) uniform Uniforms {
    mat4 projection;
    vec2 resolution;
    float highlight_addr;
};

layout(set = 0, binding = 1) uniform texture2D heatmap_texture;
layout(set = 0, binding = 2) uniform sampler heatmap_sampler;

layout(location = 0) out vec4 out_color;

void main() {
    vec4 color = texture(sampler2D(heatmap_texture, heatmap_sampler), tex_coord);
    out_color = color;
}