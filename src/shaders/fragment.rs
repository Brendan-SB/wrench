pub const MAX_LIGHTS: usize = 1024;

vulkano_shaders::shader! {
    ty: "fragment",
    src:
    "
    #version 450

    #define MAX_LIGHTS 1024

    layout(location = 0) in vec3 v_normal;
    layout(location = 1) in vec2 tex_coord;

    layout(location = 0) out vec4 f_color;

    layout(set = 0, binding = 1) uniform sampler2D tex;

    void main() {
        f_color = texture(tex, tex_coord);
    }
    "
}
