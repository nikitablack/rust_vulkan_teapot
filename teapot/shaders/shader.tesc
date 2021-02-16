#version 450

layout(push_constant) uniform PushConst
{
    float tesselationValue;
};

layout(vertices = 16) out;

in gl_PerVertex
{
    vec4 gl_Position;
} gl_in[gl_MaxPatchVertices];

void main()
{
    if (gl_InvocationID == 0)
    {
        gl_TessLevelInner[0] = tesselationValue;
        gl_TessLevelInner[1] = tesselationValue;

        gl_TessLevelOuter[0] = tesselationValue;
        gl_TessLevelOuter[1] = tesselationValue;
        gl_TessLevelOuter[2] = tesselationValue;
        gl_TessLevelOuter[3] = tesselationValue;
    }

    gl_out[gl_InvocationID].gl_Position = gl_in[gl_InvocationID].gl_Position;
}
