#version 330 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 texture_coords_in;

uniform mat4 mvp;
out vec2 texture_coords;

void main() {
    gl_Position = mvp * vec4(position, 1.0);
    texture_coords = texture_coords_in;
}