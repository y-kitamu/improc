#version 460

in vec3 Color;

layout(location = 0) out vec4 color;

void main() {
    color = vec4(Color, 1.0);
}
