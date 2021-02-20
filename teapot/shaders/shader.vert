#version 450

struct ControlPoint
{
    float x;
    float y;
    float z;
};

layout(set = 0, binding = 0) readonly buffer ControlPointBuffer
{
    ControlPoint data[];
} controlPointBuffer;

out gl_PerVertex
{
    vec4 gl_Position;
};

void main()
{
    ControlPoint cp = controlPointBuffer.data[gl_VertexIndex];

    gl_Position = vec4(cp.x, cp.y, cp.z, 1.0);
}