#version 460

in vec3 iPosition;
in vec3 iColor;

uniform mat4 uModel;
uniform mat4 uView;
uniform mat4 uProjection;
uniform float uPointSize;

out vec3 Color;

void main() {
    gl_PointSize = uPointSize;
    Color = iColor;
    gl_Position = uProjection * uView * uModel * vec4(iPosition, 1.0);
}
