// SPDX-FileCopyrightText: © 2025 - 2026 Elias Steininger <elias.st4600@gmail.com> and Project Contributors (see CONTRIBUTORS.md)
// SPDX-License-Identifier: GPL-3.0-or-later

#version 330 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 texture_coords_in;
layout(location = 2) in float shading_value;

uniform mat4 mvp;
out vec2 texture_coords;
out float shading;

void main() {
    gl_Position = mvp * vec4(position, 1.0);
    texture_coords = texture_coords_in;
    shading = shading_value;
}