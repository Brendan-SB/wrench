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

    struct Light {
        vec3 position;
        float intensity;
    };

    struct LightArray {
        uint size;
        Light array[MAX_LIGHTS];
    };

    layout(set = 0, binding = 1) uniform sampler2D tex;
    layout(set = 0, binding = 2) uniform Data {
        LightArray lights;
    } uniforms;

    void main() {
        float brightness = 0.0;

        for (uint i = 0; i < uniforms.lights.size; i++) {
            brightness += dot(normalize(v_normal), normalize(uniforms.lights.array[i].position)) * uniforms.lights.array[i].intensity;
        }

        f_color = texture(tex, tex_coord) * brightness;
    }
    "
}
