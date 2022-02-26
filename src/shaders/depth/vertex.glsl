#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec3 normal;

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

    gl_Position = uniforms.proj * world_view * uniforms.scale * vec4(position, 1.0);
}
