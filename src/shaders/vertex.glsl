#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec3 normal;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec2 tex_coords;
layout(location = 2) out vec4 f_pos;
layout(location = 3) out mat4 g_r;

layout(set = 0, binding = 0) uniform Data {
    mat4 proj;
    mat4 scale;
    mat4 translation;
    mat4 rotation;
    mat4 cam_translation;
    mat4 cam_rotation;
} uniforms;

void main() {
    mat4 transform = inverse(uniforms.rotation * uniforms.translation);
    mat4 cam_transform = inverse(uniforms.cam_rotation) * uniforms.cam_translation;

    v_normal = normal;
    tex_coords = uv;
    f_pos = transform * uniforms.scale * vec4(position, 1.0);
    g_r = uniforms.rotation;
    gl_Position = uniforms.proj * (cam_transform * f_pos);
}
