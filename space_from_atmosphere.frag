#version 410

uniform sampler2D s2Test;

in vec3 secondaryColor;
in vec2 fragTexCoord;

out vec4 finalColor;

void main ()
{
	finalColor = vec4(secondaryColor, 1.0) * texture2D(s2Test, fragTexCoord);
	//gl_FragColor = gl_SecondaryColor;
	finalColor = vec4(1.0, 0.0, 0.0, 1.0);
}
