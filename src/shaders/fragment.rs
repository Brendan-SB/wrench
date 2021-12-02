vulkano_shaders::shader! {
    ty: "fragment",
    src:
    "
    #version 450

    layout(location = 0) in vec3 v_normal;
    layout(location = 1) in vec2 tex_pos;

    layout(location = 0) out vec4 f_color;

    struct Light {
        vec3 position;
        float intensity;
    };

    layout(set = 0, binding = 1) uniform sampler2D tex;

    void main() {
        f_color = texture(tex, tex_pos);
    }
    "
}
