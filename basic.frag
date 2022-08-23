#version 410

#define NEAR 0.1

layout(std140) uniform;

uniform sampler2D tex;
uniform sampler2D shadowMap;
uniform sampler1D distribution;

uniform vec3 texelSize;
uniform vec3 lightPosition;
uniform vec3 lightColor;
uniform float ambientIntensity;
uniform float numBlockerSearchSamples = 2;
uniform float uvLightSize = 1;
uniform float frustumSize;

in vec2 fragTexCoord;
in vec3 surfaceNormal;
in vec4 worldPos;
in vec4 fragPosLightSpace;

out vec4 finalColor;

vec2 RandomDirection(float u) {
    return texture(distribution, u).xy * 2 - vec2(1);
}

float sample_shadow_map_pcf(sampler2D shadowMap, vec2 coords, vec2 texel_size, float uvRadius, float currentDepth, float bias) {
    float result = 0.0f;

    float samples = int(uvRadius / 0.9);
    samples = samples > 40 ? 40 : samples;
    samples = samples < 1 ? 2 : samples;
    float samples_start = samples / 2.0f;
    int count = 0;

    for(float y = -samples_start; y <= samples_start; y += 1.0f) {
        count++;
        for(float x = -samples_start; x <= samples_start; x += 1.0f) {

            vec2 coordsOffset = vec2(x, y) * texel_size * 2;
            float pcfDepth = texture(shadowMap, coords + coordsOffset).r;
            result += currentDepth - bias * 1.4 > pcfDepth ? 1.0 : 0.0;

        }
    }

    count = count > 0 ? count : 1;
    result /= (count * count);

    // keep the shadow at 0.0 when outside the far_plane region of the light's frustum.
    if(currentDepth > 1.0)
        result = 0.0;

    return result;
}

float SearchWidth(float uvLightSize, float receiverDistance) {
    return uvLightSize * (receiverDistance - NEAR) / receiverDistance;
}

float FindBlockerDistance_DirectionalLight(vec3 shadowCoords, sampler2D shadowMap, float uvLightSize, float compare, vec2 texel_size) {
    int blockers = 0;
    float avgBlockerDistance = 0;
    float searchWidth = SearchWidth(uvLightSize, shadowCoords.z);
    for(int i = 0; i < numBlockerSearchSamples; i++) {
        vec2 coordsOffset = RandomDirection(float(i) / float(numBlockerSearchSamples)) * texel_size;
        float z = texture(shadowMap, shadowCoords.xy + coordsOffset * (searchWidth / numBlockerSearchSamples)).r;
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

float sample_shadow_map_pcss(sampler2D shadowMap, vec3 shadowCoords, float uvLightSize, float compare, vec2 texel_size, float currentDepth, float bias) {
	// blocker search
    float blockerDistance = FindBlockerDistance_DirectionalLight(shadowCoords, shadowMap, uvLightSize, compare, texel_size);
    if(blockerDistance == -1)
        return 1;		

	// penumbra estimation
    float penumbraWidth = ((shadowCoords.z - blockerDistance) * uvLightSize) / blockerDistance;

	// percentage-close filtering
    float uvRadius = penumbraWidth * NEAR / shadowCoords.z;
    return 1 - sample_shadow_map_pcf(shadowMap, shadowCoords.xy, texel_size, uvRadius, currentDepth, bias);
}

float compute_shadow(vec4 fragPosLightSpace, float uvLightSize, float bias) {
    vec3 shadowMapCoords = (fragPosLightSpace.xyz / fragPosLightSpace.w);

    return sample_shadow_map_pcss(shadowMap, shadowMapCoords, uvLightSize, shadowMapCoords.z - bias, texelSize.xy, shadowMapCoords.z, bias);
}

void main() {

    vec4 textureSample = texture(tex, fragTexCoord);
    vec2 samplerSize = textureSize(tex, 0);

    if(samplerSize.x > 1 && samplerSize.y > 1 && textureSample.a < 0.1)
        discard;

    vec4 AmbientColor = textureSample * vec4(lightColor, 1.0) *
        ambientIntensity;

    vec3 toLightVector = lightPosition - worldPos.xyz;
    vec3 unitNormal = normalize(surfaceNormal);
    vec3 unitLightPosition = normalize(toLightVector);

    float nDotL = dot(unitNormal, unitLightPosition);
    float brightness = max(nDotL, 0.0);

    vec4 DiffuseColor = textureSample * vec4(lightColor, 1.0) * brightness;

    float bias = max(0.05 * (1.0 - dot(unitNormal, unitLightPosition)), 0.001);

    float shadow = compute_shadow(fragPosLightSpace, uvLightSize / frustumSize, bias);

    AmbientColor += shadow;

    finalColor = (AmbientColor * DiffuseColor);
}
