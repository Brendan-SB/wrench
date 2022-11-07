#version 450
#define MAX_LIGHTS 256

struct Light {
    mat4 position;
    mat4 rotation;
    mat4 proj;
    vec3 color;
    bool directional;
    float intensity;
    float cutoff;
    float outer_cutoff;
    float attenuation;
};

struct LightArray {
    uint len;
    Light array[MAX_LIGHTS];
};

layout(location = 0) in vec3 normal;
layout(location = 1) in vec2 tex_coord;
layout(location = 2) in vec4 f_pos;
layout(location = 3) in mat4 global_rotation;

layout(location = 0) out vec4 f_color;

layout(set = 1, binding = 0) uniform sampler2D tex;

layout(set = 0, binding = 1) uniform Data {
    bool lit;
    vec4 color;
    float ambient;
    float diff_strength;
    float spec_strength;
    uint spec_power;
    LightArray lights;
} uniforms;

vec4 light_calculations(vec3 norm) {
    vec4 brightness = vec4(uniforms.ambient);

    for (uint i = 0; i < uniforms.lights.len; i++) {
        Light light = uniforms.lights.array[i];

        vec3 f_pos_dif = vec3((-(light.position * vec4(vec3(0.0), 1.0)) - f_pos).xyz);
        vec3 light_dir = normalize(f_pos_dif);

        float dist = length(f_pos_dif);
        float attenuation = 1.0 / (light.attenuation * pow(dist, 2));
        float edge_softness = 1.0;

        if (light.directional) {
          float theta = dot(light_dir, -normalize(vec3(light.rotation * vec4(0.0, 0.0, 1.0, 1.0))));

          if (theta > light.outer_cutoff) {
            float epsilon = light.cutoff - light.outer_cutoff;
            
            edge_softness = clamp((theta - light.outer_cutoff) / epsilon, 0.0, 1.0);
          } else {
            continue;
          }
        }

        float diff = max(dot(norm, light_dir), 0.0) * uniforms.diff_strength;

        brightness += diff * light.intensity * vec4(light.color, 1.0) * attenuation * edge_softness;
    }

    return brightness;
}

void main() {
    vec4 tex_color = texture(tex, tex_coord) * uniforms.color;

    f_color = tex_color;

    if (uniforms.lit) {
      vec3 norm = normalize((inverse(global_rotation) * vec4(normal, 1.0)).xyz);

      vec4 brightness = light_calculations(norm);

      f_color *= vec4(brightness.xyz, 1.0);
    }
}
