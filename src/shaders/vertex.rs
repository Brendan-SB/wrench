vulkano_shaders::shader! {
    ty: "vertex",
    src:
    "
    #version 450

    layout(location = 0) in vec3 position;
    layout(location = 1) in vec2 uv;
    layout(location = 2) in vec3 normal;

    layout(location = 0) out vec3 v_normal;
    layout(location = 1) out vec2 o_tex_coords;

    layout(set = 0, binding = 0) uniform Data {
        mat4 world;
        mat4 view;
        mat4 proj;
        mat4 translation;
        mat4 cam_translation;
    } uniforms;

    void main() {
        mat4 worldview = uniforms.view * uniforms.world;

        v_normal = transpose(inverse(mat3(worldview))) * normal;
        
        mat4 cam_transform = worldview * uniforms.cam_translation;
        vec4 pos = uniforms.proj * uniforms.translation * cam_transform * vec4(position, 1.0);

        gl_Position = pos;
        o_tex_coords = uv;
    }
    "
}
