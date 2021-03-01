#version 450

layout (quads, fractional_odd_spacing, cw) in;

struct PatchData
{
    mat4 transform;
    vec4 color;
};

layout(set = 0, binding = 1) readonly buffer StorageBuffer
{
    PatchData patchData[];
};

layout(set = 0, binding = 2) uniform UniformBuffer
{
    mat4 mvpMatrix;
};

layout (location = 0) out vec3 outColor;

vec4 bernsteinBasis(float t)
{
    float invT = 1.0f - t;

    return vec4(invT * invT * invT,     // (1-t)^3
                3.0f * t * invT * invT, // 3t(1-t)^2
                3.0f * t * t * invT,    // 3t2(1-t)
                t * t * t);             // t3
}

vec4 evaluateBezier(vec4 basisU, vec4 basisV)
{
    vec4 value = vec4(0.0, 0.0, 0.0, 0.0);

    value = basisV.x * (gl_in[0].gl_Position * basisU.x + gl_in[1].gl_Position * basisU.y + gl_in[2].gl_Position * basisU.z + gl_in[3].gl_Position * basisU.w);
    value += basisV.y * (gl_in[4].gl_Position * basisU.x + gl_in[5].gl_Position * basisU.y + gl_in[6].gl_Position * basisU.z + gl_in[7].gl_Position * basisU.w);
    value += basisV.z * (gl_in[8].gl_Position * basisU.x + gl_in[9].gl_Position * basisU.y + gl_in[10].gl_Position * basisU.z + gl_in[11].gl_Position * basisU.w);
    value += basisV.w * (gl_in[12].gl_Position * basisU.x + gl_in[13].gl_Position * basisU.y + gl_in[14].gl_Position * basisU.z + gl_in[15].gl_Position * basisU.w);
    value.w = 1.0;

    return value;
}

void main(void)
{
    vec4 basisU = bernsteinBasis(gl_TessCoord.x);
    vec4 basisV = bernsteinBasis(gl_TessCoord.y);

    vec4 localPos = evaluateBezier(basisU, basisV);

    gl_Position = mvpMatrix * patchData[gl_PrimitiveID].transform * localPos;

    outColor = patchData[gl_PrimitiveID].color.xyz;
}