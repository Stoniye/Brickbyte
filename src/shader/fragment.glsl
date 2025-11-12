#version 330 core

in vec2 texture_coords;
out vec4 out_color;

uniform sampler2D texture_atlas;

void main() {
    out_color = texture(texture_atlas, texture_coords);
}