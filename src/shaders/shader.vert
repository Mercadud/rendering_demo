#version 450

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec3 colour;
layout (location = 3) in mat4 object_matrix;

layout (location = 0) out vec3 v_normal;
layout (location = 1) out vec3 v_colour;


layout (set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
    bool lev_2;
} uniforms;

void main() {
    vec3 transformed_direction = vec3(object_matrix * vec4(position, 1.0));
    v_normal = vec3(object_matrix * vec4(normal, 0.0));
    v_colour = vec3(colour);
    if (uniforms.lev_2) {
        gl_Position = uniforms.proj *
        uniforms.view *
        vec4(transformed_direction, 1.0);
    } else {
        gl_Position = vec4(transformed_direction, 1.0);
    }
}