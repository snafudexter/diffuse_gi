#version 410

#define NEAR 0.1

layout(std140) uniform;

uniform sampler2D tex;
uniform sampler2D shadowMap;
uniform sampler1D distribution;

uniform vec3 eyePosition;
uniform vec3 texelSize;
uniform vec3 lightPosition;
uniform vec3 lightColor;
uniform float ambientIntensity;
uniform float shadowBias;
uniform float numBlockerSearchSamples = 16;
uniform float uvLightSize = 1;
uniform float frustumSize;

in vec2 fragTexCoord;
in vec3 surfaceNormal;
in vec4 worldPos;
in vec4 fragPosLightSpace;

out vec4 finalColor;

vec2 poissonDisk[16] = vec2[](vec2(-0.94201624, -0.39906216), vec2(0.94558609, -0.76890725), vec2(-0.094184101, -0.92938870), vec2(0.34495938, 0.29387760), vec2(-0.91588581, 0.45771432), vec2(-0.81544232, -0.87912464), vec2(-0.38277543, 0.27676845), vec2(0.97484398, 0.75648379), vec2(0.44323325, -0.97511554), vec2(0.53742981, -0.47373420), vec2(-0.26496911, -0.41893023), vec2(0.79197514, 0.19090188), vec2(-0.24188840, 0.99706507), vec2(-0.81409955, 0.91437590), vec2(0.19984126, 0.78641367), vec2(0.14383161, -0.14100790));

vec2 RandomDirection(sampler1D distribution, float u)
{
   return texture(distribution, u).xy * 2 - vec2(1);
}

float sample_shadow_map(sampler2D shadowMap, vec2 coords, float compare) {
    return step(compare, texture2D(shadowMap, coords.xy).r);
}

float sample_shadow_map_pcf(sampler1D distribution, sampler2D shadowMap, vec2 coords, float compare, vec2 texel_size, float uvRadius) {
    float result = 0.0f;

    //const float samples = 5.0f;
    // const float samples_start = (samples - 1.0f) / 2.0f;
    // const float samples_squared = samples * samples;

    // for(float y = -samples_start; y < samples_start; y += 1.0f) {
    //     for(float x = -samples_start; x < samples_start; x += 1.0f) {

    //         vec2 coordsOffset = vec2(x, y) * texel_size * uvRadius;
    //         result += sample_shadow_map(shadowMap, coords + coordsOffset, compare);

    //     }
    // }

    // return result / samples_squared;

    for(int i = 0; i < numBlockerSearchSamples; i++) {
        vec2 coordsOffset = RandomDirection(distribution, float(float(i) / float(numBlockerSearchSamples))) * texel_size * uvRadius;
        float z = texture2D(shadowMap, coords.xy + coordsOffset).r;
        result += z < compare ? 1.0 : 0.0;
    }

    return result / numBlockerSearchSamples;
}

float SearchWidth(float uvLightSize, float receiverDistance) {
    return uvLightSize * (receiverDistance - NEAR) / eyePosition.z;
}

float FindBlockerDistance_DirectionalLight(sampler1D distribution, vec3 shadowCoords, sampler2D shadowMap, float uvLightSize, float compare, vec2 texel_size) {
    int blockers = 0;
    float avgBlockerDistance = 0;
    float searchWidth = SearchWidth(uvLightSize, shadowCoords.z);
    for(int i = 0; i < numBlockerSearchSamples; i++) {
        vec2 coordsOffset = RandomDirection(distribution, float(i) / float(numBlockerSearchSamples)) * texel_size;
        float z = texture2D(shadowMap, shadowCoords.xy + coordsOffset * searchWidth).r;
        if(z < (compare)) {
            blockers++;
            avgBlockerDistance += z;
        }
    }
    if(blockers > 0)
        return avgBlockerDistance / blockers;
    else
        return -1;
}

float sample_shadow_map_pcss(sampler1D distribution, sampler2D shadowMap, vec3 shadowCoords, float uvLightSize, float compare, vec2 texel_size) {
	// blocker search
    float blockerDistance = FindBlockerDistance_DirectionalLight(distribution,shadowCoords, shadowMap, uvLightSize, compare, texel_size);
    if(blockerDistance == -1)
        return 1;		

	// penumbra estimation
    float penumbraWidth = (shadowCoords.z - blockerDistance) / blockerDistance;

	// percentage-close filtering
    float uvRadius = penumbraWidth * uvLightSize * NEAR / shadowCoords.z;
    return 1 - sample_shadow_map_pcf(distribution, shadowMap, shadowCoords.xy, compare, texel_size, uvRadius);
}

float compute_shadow(vec4 fragPosLightSpace, float uvLightSize) {
    vec3 shadowMapCoords = (fragPosLightSpace.xyz / fragPosLightSpace.w);

    return sample_shadow_map_pcss(distribution, shadowMap, shadowMapCoords, uvLightSize, shadowMapCoords.z - shadowBias, texelSize.xy);

    // return sample_shadow_map_pcf(shadowMap, shadowMapCoords.xy, shadowMapCoords.z - shadowBias, texelSize.xy, 1.0);
}

void main() {

    vec4 textureSample = texture2D(tex, fragTexCoord);

    vec4 AmbientColor = textureSample * vec4(lightColor, textureSample.a) *
        ambientIntensity;

    vec3 toLightVector = lightPosition - worldPos.xyz;
    vec3 unitNormal = normalize(surfaceNormal);
    vec3 unitLightPosition = normalize(toLightVector);

    float nDotL = dot(unitNormal, unitLightPosition);
    float brightness = max(nDotL, 0.0);

    vec4 DiffuseColor = textureSample * vec4(lightColor, textureSample.a) * 0.4 * brightness;

    float shadow = compute_shadow(fragPosLightSpace, uvLightSize / frustumSize);

    finalColor = (AmbientColor + DiffuseColor * shadow);
}
