#version 450

#define MAX_LIGHTS 1024

struct Light {
    mat4 position;
    mat4 rotation;
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
layout(location = 2) in vec4 pos;
layout(location = 3) in vec3 f_pos;
layout(location = 4) in mat3 global_rotation;
layout(location = 7) in mat3 cam_translation;

layout(location = 0) out vec4 f_color;

layout(set = 1, binding = 0) uniform sampler2D tex;
layout(set = 1, binding = 1) uniform sampler2D shadow_buffer;

layout(set = 0, binding = 1) uniform Data {
    vec4 color;
    float ambient;
    float diff_strength;
    float spec_strength;
    uint spec_power;
    LightArray lights;
} uniforms;

vec4 light_calculations(vec3 norm, mat4 cam_offset) {
    vec4 brightness = vec4(uniforms.ambient);

    for (uint i = 0; i < uniforms.lights.len; i++) {
        Light light = uniforms.lights.array[i];

        vec4 pos_light_space = inverse(light.position) * pos * 0.5 + 0.5;

        vec3 shadow_coord = pos_light_space.xyz / pos_light_space.w;

        float shadow = texture(shadow_buffer, shadow_coord.xy * 0.5).z;

        vec3 f_pos_dif = vec3((cam_offset * light.position * vec4(vec3(0.0), 1.0)).xyz - f_pos);
        vec3 light_dir = normalize(f_pos_dif);
        vec3 view_dir = -normalize(f_pos);

        float dist = length(f_pos_dif);
        float attenuation = 1.0 / (light.attenuation * pow(dist, 2));
        float edge_softness = 1.0;

        if (light.directional) {
          float theta = dot(light_dir, -normalize(vec3(cam_offset * inverse(light.rotation) * vec4(0.0, 0.0, 1.0, 1.0))));

          if (theta > light.outer_cutoff) {
            float epsilon = light.cutoff - light.outer_cutoff;
            
            edge_softness = clamp((theta - light.outer_cutoff) / epsilon, 0.0, 1.0);
          } else {
            continue;
          }
        }

        vec3 reflect_dir = reflect(norm, light_dir);

        float diff = max(dot(norm, light_dir), 0.0) * uniforms.diff_strength;
        float spec = pow(max(dot(view_dir, reflect_dir), 0.0), uniforms.spec_power) * uniforms.spec_strength;

        brightness += (diff + spec) * light.intensity * vec4(light.color, 1.0) * attenuation * edge_softness - shadow;
    }

    return brightness;
}

void main() {
    vec4 tex_color = texture(tex, tex_coord) * uniforms.color;
    vec3 norm = normalize(global_rotation * normal);

    mat4 cam_offset = -mat4(cam_translation);

    vec4 brightness = light_calculations(norm, cam_offset);

    f_color = tex_color * vec4(brightness.xyz, 1.0);
}
