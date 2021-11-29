vulkano_shaders::shader! {
    ty: "vertex",
    src:
    "
    #version 450

    struct Rotation {
        mat4 x;
        mat4 y;
        mat4 z;
    };

    struct Transform {
        Rotation rotation;
        vec3 position;
    };

    layout(location = 0) in vec3 position;
    layout(location = 1) in vec3 normal;
    layout(location = 0) out vec3 v_normal;

    layout(set = 0, binding = 0) uniform Data {
        mat4 view;
        mat4 proj;
        Transform transform;
    } uniforms;

    void main() {
        v_normal = transpose(inverse(mat3(uniforms.view))) * normal;
        gl_Position = uniforms.view
            * uniforms.transform.rotation.x
            * uniforms.transform.rotation.y
            * uniforms.transform.rotation.z 
            * uniforms.proj
            * vec4(position - uniforms.transform.position, 1.0);
    }
    "
}
