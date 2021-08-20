#version 300 es

precision mediump float;
out vec4 outColor;

uniform sampler2D uSampler;

in vec3 pass_normal;
in vec2 pass_tex;

void main() {
    outColor = vec4(texture(uSampler, pass_tex.xy));
}
