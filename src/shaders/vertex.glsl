#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec3 normal;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec2 tex_coords;
layout(location = 2) out vec4 f_pos;
layout(location = 3) out mat3 g_r;

layout(set = 0, binding = 0) uniform Data {
    mat4 proj;
    mat4 scale;
    mat4 transform;
    mat4 cam_transform;
} uniforms;

void main() {
    mat4 global_transform = inverse(uniforms.transform);
    mat4 world_view = uniforms.cam_transform * global_transform;

    v_normal = normal;
    tex_coords = uv;
    f_pos = global_transform * uniforms.scale * vec4(position, 1.0);
    g_r = mat3(global_transform);
    gl_Position = uniforms.proj * world_view * uniforms.scale * vec4(position, 1.0);
}
