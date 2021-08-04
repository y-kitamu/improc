#version 140

in vec2 TexCoords;

uniform sampler2D uImageTexture;

void main() {
    gl_FragColor = vec4(texture(uImageTexture, TexCoords).rgb, 1.0);
}
