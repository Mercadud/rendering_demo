#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

layout (location = 2) in vec3 transform;
layout (location = 3) in vec3 colour;
layout (location = 4) in float scale;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec3 v_colour;

layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
    bool lev_2;
} uniforms;

void main() {
    v_colour = colour;
    vec3 transformed_position = (scale * position) + transform;
    if (uniforms.lev_2) {
        mat4 worldview = uniforms.view * uniforms.world;
        v_normal = transpose(inverse(mat3(worldview))) * normal;
        gl_Position = uniforms.proj * worldview * vec4(transformed_position, 1.0);
    }
    else {
        v_normal = normal;
        gl_Position = vec4(position.x, -position.y, position.z, 1.0);
    }
}