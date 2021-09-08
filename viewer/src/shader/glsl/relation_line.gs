#version 460

layout (lines) in;

layout (line_strip, max_vertices=2) out;

void main() {
    if (gl_in[0].gl_Position.x < gl_in[1].gl_Position.x &&
        gl_in[0].gl_Position.x * gl_in[1].gl_Position.x < 0 &&
        abs(gl_in[0].gl_Position.x) < 1.0 &&
        abs(gl_in[0].gl_Position.y) < 1.0 &&
        abs(gl_in[1].gl_Position.x) < 1.0 &&
        abs(gl_in[1].gl_Position.y) < 1.0) {
        for (int i = 0; i < 2; i++) {
            gl_Position = gl_in[i].gl_Position;
            gl_PointSize = gl_in[i].gl_PointSize;
            EmitVertex();
        }
    }
    EndPrimitive();
}
