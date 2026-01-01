// SPDX-FileCopyrightText: © 2025 - 2026 Elias Steininger <elias.st4600@gmail.com> and Project Contributors (see CONTRIBUTORS.md)
// SPDX-License-Identifier: GPL-3.0-or-later

#version 330 core

in vec2 texture_coords;
in float shading;
out vec4 out_color;

uniform sampler2D texture_atlas;

void main() {
    vec4 tex_color = texture(texture_atlas, texture_coords);
    if (tex_color.a < 0.1) discard;

    out_color = vec4(tex_color.rgb * shading, tex_color.a);
}