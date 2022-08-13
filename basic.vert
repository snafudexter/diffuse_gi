#version 460

uniform mat4 camera;
uniform mat4 model;
uniform mat4 depthBiasMatrix;


layout (location = 0) in vec3 position;
layout (location = 1) in vec2 tex_coord;
layout (location = 2) in vec3 normal;

out vec3 fragVert;
out vec2 fragTexCoord;
out vec3 fragNormal;
out vec4 fragPosLightSpace;

void main() {
                // Pass some variables to the fragment shader
    fragTexCoord = tex_coord;
    fragNormal = normal;
    fragVert = position;
    fragPosLightSpace = depthBiasMatrix * vec4(fragVert, 1.0);

                // Apply all matrix transformations to vert
    gl_Position = camera * model * vec4(position, 1);
}