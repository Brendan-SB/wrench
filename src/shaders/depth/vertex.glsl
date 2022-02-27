#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec3 normal;

layout(set = 0, binding = 0) uniform Data {
    mat4 proj;
    mat4 scale;
    mat4 transform;
    mat4 cam_transform;
} uniforms;

void main() {
    mat4 world_view = uniforms.cam_transform * inverse(uniforms.transform);

    gl_Position = uniforms.proj * world_view * uniforms.scale * vec4(position, 1.0);
}
