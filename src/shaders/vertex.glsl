#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec3 normal;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec2 tex_coords;
layout(location = 2) out mat3 cv;

layout(set = 0, binding = 0) uniform Data {
    mat4 rotation;
    mat4 cam_rotation;
    mat4 proj;
    mat4 translation;
    mat4 cam_translation;
    mat4 scale;
    mat4 transform;
    mat4 cam_transform;
    mat4 worldview;
} uniforms;

void main() {
    v_normal = normalize(transpose(inverse(mat3(uniforms.transform))) * normal);
    tex_coords = uv;
    cv = transpose(inverse(mat3(uniforms.cam_translation)));
    gl_Position = uniforms.proj * uniforms.worldview * uniforms.scale * vec4(position, 1.0);
}
