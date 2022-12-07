#version 450

layout (location = 0) in vec3 v_normal;
layout (location = 1) in vec3 v_colour;

layout (location = 0) out vec4 f_color;

const vec3 LIGHT = vec3(0.0, 0.0, 1.0);


layout (set = 0, binding = 1) uniform Data {
    bool lighting;
} uniforms;

void main() {
    if (uniforms.lighting == true) {
        float brightness = dot(normalize(v_normal), normalize(LIGHT));
        vec3 dark_color = vec3(0.0, 0.0, 0.0);
        vec3 regular_color = v_colour;

        f_color = vec4(mix(dark_color, regular_color, brightness), 1.0);
    } else {
        f_color = vec4(v_colour, 1.0);
    }
}