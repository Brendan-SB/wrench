vulkano_shaders::shader! {
    ty: "vertex",
    src:
    "
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
    } uniforms;

    void main() {
        mat4 cam_transform = uniforms.cam_rotation * uniforms.cam_translation;
        mat4 transform = uniforms.translation * uniforms.rotation * uniforms.scale;

        v_normal = normalize(transpose(inverse(mat3(transform))) * normal);

        mat4 worldview = cam_transform * transform;

        cv = transpose(inverse(mat3(uniforms.cam_translation)));

        vec4 pos = uniforms.proj * worldview * vec4(position, 1.0);

        tex_coords = uv;
        gl_Position = pos;
    }
    "
}
