#version 410

layout(std140) uniform;

uniform sampler2D tex;

uniform vec3 LightColor;
uniform float AmbientIntensity;
uniform float DiffuseIntensity;

in vec2 fragTexCoord;
in vec3 surfaceNormal;
in vec4 worldPos;
uniform vec3 LightPosition;

out vec4 finalColor;

void main() {
    vec4 AmbientColor = vec4(LightColor, 1.0f) *
        AmbientIntensity;

    vec3 toLightVector = LightPosition - worldPos.xyz;
    vec3 unitNormal = normalize(surfaceNormal);
    vec3 unitLightPosition = normalize(toLightVector);

    float nDotL = dot(unitNormal, unitLightPosition);
    float brightness = max(nDotL, 0.0);

    vec4 DiffuseColor = vec4(LightColor, 1.0f) * brightness;

    finalColor =  texture(tex, fragTexCoord) * (AmbientColor + DiffuseColor);
}