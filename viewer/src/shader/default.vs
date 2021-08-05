#version 140

in vec3 iPosition;
in vec2 iTexCoords;

uniform mat4 uModel;
uniform mat4 uView;
uniform mat4 uProjection;

out vec2 TexCoords;

void main() {
    TexCoords = iTexCoords;
    gl_Position = uProjection * uView * uModel * vec4(iPosition, 1.0);
}
