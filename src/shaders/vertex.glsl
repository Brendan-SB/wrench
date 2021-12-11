#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec3 normal;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec2 tex_coords;
layout(location = 2) out vec3 f_pos;
layout(location = 3) out mat4 cam_translation;

layout(set = 0, binding = 0) uniform Data {
    mat4 rotation;
    mat4 cam_rotation;
    mat4 proj;
    mat4 translation;
    mat4 cam_translation;
    mat4 scale;
    mat4 transform;
    mat4 cam_transform;
} uniforms;

void main() {
    mat4 global_transform = inverse(uniforms.transform);
    mat4 world_view = uniforms.cam_transform * global_transform;

    v_normal = normalize(mat3(global_transform) * normal);
    tex_coords = uv;
    f_pos = vec3(global_transform * vec4(position, 1.0));
    cam_translation = uniforms.cam_translation;
    gl_Position = uniforms.proj * world_view * uniforms.scale * vec4(position, 1.0);
}
