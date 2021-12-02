vulkano_shaders::shader! {
    ty: "vertex",
    src:
    "
    #version 450

    layout(location = 0) in vec3 position;
    layout(location = 1) in vec3 normal;

    layout(location = 0) out vec3 v_normal;
    layout(location = 1) out vec2 tex_pos;

    layout(set = 0, binding = 0) uniform Data {
        mat4 world;
        mat4 view;
        mat4 proj;
        vec3 position;
    } uniforms;

    void main() {
        mat4 worldview = uniforms.view * uniforms.world;

        v_normal = transpose(inverse(mat3(worldview))) * normal;

        vec4 pos = uniforms.proj * worldview * vec4(position + uniforms.position, 1.0);

        gl_Position = pos;
        tex_pos = vec2(pos);
    }
    "
}
