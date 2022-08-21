#version 410

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tex_coord;
layout(location = 2) in vec3 normal;

uniform mat4 view_proj;
uniform mat4 model;
uniform mat4 light_space_matrix;

out vec3 surfaceNormal;
out vec2 fragTexCoord;
out vec4 worldPos;
out vec4 fragPosLightSpace;

void main() {
    fragTexCoord = tex_coord;
    worldPos = model * vec4(position, 1);
    surfaceNormal = (model * vec4(normal, 0.0)).xyz;
    fragPosLightSpace = light_space_matrix * worldPos;
    gl_Position = view_proj * worldPos;
}