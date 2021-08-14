#version 300 es

uniform mat4 view_projection;

in vec3 position;
in vec3 norm;
in vec2 tex_coord;

out vec3 pass_normal;
out vec2 pass_tex;

void main() {
    gl_Position = view_projection * vec4(position.xyz, 1.0f);
    pass_normal = norm;
    pass_tex = tex_coord;
}