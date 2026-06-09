// Memory heatmap fragment shader
#version 450

layout(location = 0) in vec2 frag_tex_coord;

layout(set = 0, binding = 0) uniform Uniforms {
    mat4 projection;
    vec2 resolution;
    float highlight_addr;
};

layout(set = 0, binding = 1) uniform texture2D heatmap_texture;
layout(set = 0, binding = 2) uniform sampler heatmap_sampler;

layout(location = 0) out vec4 out_color;

void main() {
    vec4 color = texture(sampler2D(heatmap_texture, heatmap_sampler), frag_tex_coord);
    
    // Color mapping for memory access types
    // R: read access intensity
    // G: write access intensity  
    // B: execution intensity
    // A: unused
    
    vec3 final_color;
    final_color.r = color.r * 0.8; // Read: blue-ish
    final_color.g = color.g * 0.8; // Write: red-ish
    final_color.b = color.b * 0.8; // Execute: green-ish
    
    // Mix based on dominant access type
    float total = color.r + color.g + color.b;
    if (total > 0.0) {
        final_color = vec3(
            color.g / total * 0.8 + 0.1,  // More red for writes
            color.b / total * 0.6 + 0.1,  // More green for execute
            color.r / total * 0.8 + 0.1   // More blue for reads
        );
    } else {
        final_color = vec3(0.05, 0.05, 0.08); // Unused memory
    }
    
    out_color = vec4(final_color, 1.0);
}