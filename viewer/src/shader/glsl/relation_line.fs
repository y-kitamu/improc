#version 460

uniform vec3 uColor;

layout(location = 0) out vec4 color;

void main() {
    color = vec4(uColor.rgb, 1.0);
}
