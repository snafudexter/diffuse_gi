#version 410

layout(std140) uniform;

uniform sampler2D tex;
uniform sampler2DShadow shadowMap;

uniform vec3 LightColor;
uniform float AmbientIntensity;

in vec2 fragTexCoord;
in vec3 surfaceNormal;
in vec4 worldPos;
in vec4 fragPosLightSpace;
uniform vec3 LightPosition;

out vec4 finalColor;

vec2 poissonDisk[16] = vec2[](vec2(-0.94201624, -0.39906216), vec2(0.94558609, -0.76890725), vec2(-0.094184101, -0.92938870), vec2(0.34495938, 0.29387760), vec2(-0.91588581, 0.45771432), vec2(-0.81544232, -0.87912464), vec2(-0.38277543, 0.27676845), vec2(0.97484398, 0.75648379), vec2(0.44323325, -0.97511554), vec2(0.53742981, -0.47373420), vec2(-0.26496911, -0.41893023), vec2(0.79197514, 0.19090188), vec2(-0.24188840, 0.99706507), vec2(-0.81409955, 0.91437590), vec2(0.19984126, 0.78641367), vec2(0.14383161, -0.14100790));

float random(vec3 seed, int i) {
    vec4 seed4 = vec4(seed, i);
    float dot_product = dot(seed4, vec4(12.9898, 78.233, 45.164, 94.673));
    return fract(sin(dot_product) * 43758.5453);
}

float compute_shadow(vec4 fragPosLightSpace) {
    // perform perspective divide
    vec3 projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
    // transform to [0,1] range
    projCoords = projCoords * 0.5 + 0.5;
    // get closest depth value from light's perspective (using [0,1] range fragPosLight as coords)
    float closestDepth = texture(shadowMap, projCoords); 
    // get depth of current fragment from light's perspective
    float currentDepth = projCoords.z;
    // check whether current frag pos is in shadow
    float bias = max(0.05 * (1.0 - dot(surfaceNormal, LightPosition - worldPos.xyz)), 0.005);
    float shadow = currentDepth - bias > closestDepth ? 1.0 : 0.0;

    return shadow;
}

float compute_pcf(vec4 fragPosLightSpace, float cosTheta) {

    float visibility = 1.0;

    float bias = 0.005 * tan(acos(cosTheta));
    bias = clamp(bias, 0, 0.01);

    vec3 projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
    // transform to [0,1] range
    projCoords = projCoords * 0.5 + 0.5;

    for(int i = 0; i < 4; i++) {
		// use either :
		//  - Always the same samples.
		//    Gives a fixed pattern in the shadow, but no noise
        //int index = i;
		//  - A random sample, based on the pixel's screen location. 
		//    No banding, but the shadow moves with the camera, which looks weird.
        //int index = int(16.0 * random(gl_FragCoord.xyy, i)) % 16;
		//  - A random sample, based on the pixel's position in world space.
		//    The position is rounded to the millimeter to avoid too much aliasing
		 int index = int(16.0*random(floor(worldPos.xyz*1000.0), i))%16;

		// being fully in the shadow will eat up 4*0.2 = 0.8
		// 0.2 potentially remain, which is quite dark.
        visibility -= 0.2 * (1.0 - texture(shadowMap, vec3(projCoords.xy + poissonDisk[index] / 1000.0, (projCoords.z - bias) / 1.0)));
    }

    return visibility;

}

void main() {
    vec4 AmbientColor = vec4(LightColor, 1.0f) *
        AmbientIntensity;

    vec3 toLightVector = LightPosition - worldPos.xyz;
    vec3 unitNormal = normalize(surfaceNormal);
    vec3 unitLightPosition = normalize(toLightVector);

    float nDotL = dot(unitNormal, unitLightPosition);
    float brightness = max(nDotL, 0.0);

    vec4 DiffuseColor = texture(tex, fragTexCoord) * vec4(LightColor, 1.0f) * brightness;

    float cosTheta = clamp(nDotL, 0, 1);

    float shadow = compute_pcf(fragPosLightSpace, cosTheta);

    finalColor = (AmbientColor + shadow * DiffuseColor * cosTheta);
}