#version 460

in vec3 iPosition;
in float iIdx;

uniform mat4 uModel[2];
uniform mat4 uView[2];
uniform mat4 uProjection[2];

void main() {
    int i = int(iIdx);
    gl_Position = uProjection[i] * uView[i] * uModel[i] * vec4(iPosition, 1.0);
}
