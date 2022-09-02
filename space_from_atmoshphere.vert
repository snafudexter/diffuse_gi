//
// Atmospheric scattering vertex shader
//
// Author: Sean O'Neil
//
// Copyright (c) 2004 Sean O'Neil
#version 410

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tex_coords;

uniform mat4 view_proj;
uniform mat4 model;
uniform vec3 v3CameraPos;		// The camera's current position
uniform vec3 v3LightPos;		// The direction vector to the light source
uniform vec3 v3InvWavelength;	// 1 / pow(wavelength, 4) for the red, green, and blue channels
uniform float fCameraHeight;	// The camera's current height
uniform float fCameraHeight2;	// fCameraHeight^2
uniform float fOuterRadius;		// The outer (atmosphere) radius
uniform float fOuterRadius2;	// fOuterRadius^2
uniform float fInnerRadius;		// The inner (planetary) radius
uniform float fInnerRadius2;	// fInnerRadius^2
uniform float fKrESun;			// Kr * ESun
uniform float fKmESun;			// Km * ESun
uniform float fKr4PI;			// Kr * 4 * PI
uniform float fKm4PI;			// Km * 4 * PI
uniform float fScale;			// 1 / (fOuterRadius - fInnerRadius)
uniform float fScaleDepth;		// The scale depth (i.e. the altitude at which the atmosphere's average density is found)
uniform float fScaleOverScaleDepth;	// fScale / fScaleDepth

out vec3 secondaryColor;
out vec2 fragTexCoord;

float scale(float fCos)
{
	float x = 1.0 - fCos;
	return fScaleDepth * exp(-0.00287 + x*(0.459 + x*(3.83 + x*(-6.80 + x*5.25))));
}

void main()
{
	// Get the ray from the camera to the vertex and its length
	vec3 v3Pos = position;
	vec3 v3Ray = v3Pos - v3CameraPos;
	float fFar = length(v3Ray);
	v3Ray /= fFar;

	// Calculate the farther intersection of the ray with the outer atmosphere (which is the far point of the ray passing through the atmosphere)
	float B = 2.0 * dot(v3CameraPos, v3Ray);
	float C = fCameraHeight2 - fOuterRadius2;
	float fDet = max(0.0, B*B - 4.0 * C);
	fFar = 0.5 * (-B + sqrt(fDet));

	// Calculate attenuation from the camera to the top of the atmosphere toward the vertex
	vec3 v3Start = v3CameraPos;
	float fHeight = length(v3Start);
	float fDepth = exp(fScaleOverScaleDepth * (fInnerRadius - fCameraHeight));
	float fAngle = dot(v3Ray, v3Start) / fHeight;
	float fScatter = fDepth*scale(fAngle);
	secondaryColor = exp(-fScatter * (v3InvWavelength * fKr4PI + fKm4PI));
	gl_Position = view_proj * model * vec4(position, 1.0);
	fragTexCoord = tex_coords;
}
