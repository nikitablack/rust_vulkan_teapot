#version 450

layout(push_constant) uniform PushConst
{
    vec2 scale;
    vec2 translate;
} pushConst;

struct Vertex
{
    float x;
    float y;
    float u;
    float v;
    uint rgba;
};

layout(set = 0, binding = 0) readonly buffer VertexBuffer
{
    Vertex data[];
} vertexBuffer;

layout(location = 0) out vec2 outUv;
layout(location = 1) out vec4 outColor;

void main()
{
    Vertex vertex = vertexBuffer.data[gl_VertexIndex];
    vec2 pos = vec2(vertex.x, vertex.y);
    gl_Position = vec4(pos * pushConst.scale + pushConst.translate, 0.0, 1.0);

    outUv = vec2(vertex.u, vertex.v);
    float d = 1.0 / 0xFF;
    float r = ((vertex.rgba >> 0) & 0xFF) * d;
    float g = ((vertex.rgba >> 8) & 0xFF) * d;
    float b = ((vertex.rgba >> 16) & 0xFF) * d;
    float a = ((vertex.rgba >> 24) & 0xFF) * d;
    outColor = vec4(r, g, b, a);
}