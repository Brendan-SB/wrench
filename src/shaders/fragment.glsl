#version 450

#define MAX_LIGHTS 1024

struct Light {
    vec3 position;
    vec4 color;
    float intensity;
};

struct LightArray {
    uint len;
    Light array[MAX_LIGHTS];
};

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec2 tex_coord;
layout(location = 2) in vec3 f_pos;
layout(location = 3) in mat4 cam_translation;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform sampler2D tex;

layout(set = 0, binding = 2) uniform Data {
    vec4 color;
    float ambient;
    LightArray lights;
} uniforms;

void main() {
    vec4 brightness = vec4(uniforms.ambient);

    for (uint i = 0; i < uniforms.lights.len; i++) {
        brightness += dot(v_normal, normalize(mat3(-cam_translation) * uniforms.lights.array[i].position - f_pos)) * uniforms.lights.array[i].intensity * uniforms.lights.array[i].color;
    }
    
    f_color = texture(tex, tex_coord) * uniforms.color * brightness;
}
