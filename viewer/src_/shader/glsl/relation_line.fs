#version 460

uniform vec4 uColor;

layout(location = 0) out vec4 color;

void main() {
    color = uColor;
}
