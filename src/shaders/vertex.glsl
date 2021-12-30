#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec3 normal;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec2 tex_coords;
layout(location = 2) out vec3 f_pos;
layout(location = 3) out mat3 g_t;
layout(location = 6) out mat4 cam_translation;

layout(set = 0, binding = 0) uniform Data {
    mat4 proj;
    mat4 scale;
    mat4 translation;
    mat4 rotation;
    mat4 cam_translation;
    mat4 cam_rotation;
} uniforms;

void main() {
    mat4 transform = uniforms.rotation * uniforms.translation;
    mat4 cam_transform = uniforms.cam_rotation * uniforms.cam_translation;
    mat4 global_transform = inverse(transform);
    mat4 world_view = cam_transform * global_transform;

    v_normal = normal;
    tex_coords = uv;
    f_pos = vec3(global_transform * vec4(position, 1.0));
    g_t = mat3(global_transform);
    cam_translation = uniforms.cam_translation;
    gl_Position = uniforms.proj * world_view * uniforms.scale * vec4(position, 1.0);
}
