#version 460

uniform mat4 model;
uniform sampler2D tex;
uniform sampler2D shadowMap;

uniform vec3 l_position;
uniform vec3 l_intensities;

in vec2 fragTexCoord;
in vec3 fragNormal;
in vec3 fragVert;
in vec4 fragPosLightSpace;

out vec4 finalColor;


void main() {
    finalColor = texture2D(tex, fragTexCoord);
}