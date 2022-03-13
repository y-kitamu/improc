#version 460

in vec2 TexCoords;

uniform sampler2D uImageTexture;

layout(location = 0) out vec4 color;

void main() {
    color = vec4(texture(uImageTexture, TexCoords).rgb, 1.0);
}
