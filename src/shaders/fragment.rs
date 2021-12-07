vulkano_shaders::shader! {
    ty: "fragment",
    src:
    "
    #version 450

    #define MAX_LIGHTS 1024

    struct Light {
        vec3 position;
        float intensity;
    };

    struct LightArray {
        float ambient;
        uint size;
        Light array[MAX_LIGHTS];
    };

    layout(location = 0) in vec3 v_normal;
    layout(location = 1) in vec2 tex_coord;
    layout(location = 2) in mat3 camview;

    layout(location = 0) out vec4 f_color;

    layout(set = 0, binding = 1) uniform sampler2D tex;

    layout(set = 0, binding = 2) uniform Data {
        LightArray lights;
    } uniforms;

    void main() {
        float brightness = uniforms.lights.ambient;

        for (uint i = 0; i < uniforms.lights.size; i++) {
            brightness += dot(v_normal, normalize(camview * uniforms.lights.array[i].position)) * uniforms.lights.array[i].intensity;
        }
        
        f_color = texture(tex, tex_coord) * brightness;
    }
    "
}
