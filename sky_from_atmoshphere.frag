//
// Atmospheric scattering fragment shader
//
// Author: Sean O'Neil
//
// Copyright (c) 2004 Sean O'Neil
#version 410

uniform vec3 v3LightPos;
uniform float g;
uniform float g2;

in vec3 v3Direction;
in vec3 color;
in vec3 secondaryColor;

out vec4 finalColor;

void main() {
    float fCos = dot(v3LightPos, v3Direction) / length(v3Direction);
    float fMiePhase = 1.5 * ((1.0 - g2) / (2.0 + g2)) * (1.0 + fCos * fCos) / pow(1.0 + g2 - 2.0 * g * fCos, 1.5);
    finalColor = (0.75 + 0.75 * fCos *fCos) *vec4(color, 1.0) + fMiePhase * vec4(secondaryColor, 1.0);
    finalColor.a = color.b;
}
