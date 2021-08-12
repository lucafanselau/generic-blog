#version 300 es

uniform mat4 view_projection;

in vec4 position;

void main() {
    gl_Position = view_projection * position;
}