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
    layout(location = 2) out mat3 wv;

    layout(set = 0, binding = 0) uniform Data {
        mat4 rotation;
        mat4 cam_rotation;
        mat4 proj;
        mat4 translation;
        mat4 cam_translation;
    } uniforms;

    void main() {
        mat4 worldview = uniforms.rotation * uniforms.cam_rotation * uniforms.translation * uniforms.cam_translation;

        wv = transpose(inverse(mat3(worldview)));
        v_normal = normalize(wv * normal);
        
        vec4 pos = uniforms.proj * worldview * vec4(position, 1.0);

        tex_coords = uv;
        gl_Position = pos;
    }
    "
}
