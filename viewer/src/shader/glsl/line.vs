#version 460

layout (location=0) in vec3 iPosition;
layout (location=1) in vec2 iCenter;

uniform mat4 uModel;
uniform mat4 uView;
uniform mat4 uProjection;
uniform float uScale;

void main() {
    float x = (iPosition.x - iCenter.x) * uScale + iCenter.x;
    float y = (iPosition.y - iCenter.y) * uScale + iCenter.y;
    // float x = iPosition.x * scale;
    // float y = iPosition.y * scale;
    gl_Position = uProjection * uView * uModel * vec4(x, y, iPosition.z, 1.0);
}
