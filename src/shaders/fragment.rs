vulkano_shaders::shader! {
    ty: "fragment",
    src:
    "
    #version 450

    layout(location = 0) in vec3 v_normal;
    layout(location = 1) in vec2 tex_pos;

    layout(location = 0) out vec4 f_color;

    layout(set = 0, binding = 1) uniform sampler2D tex;

    const vec3 LIGHT = vec3(0.0, 0.0, 1.0);

    void main() {
        float brightness = dot(normalize(v_normal), normalize(LIGHT));
        
        f_color = texture(tex, tex_pos) * brightness;
    }
    "
}
