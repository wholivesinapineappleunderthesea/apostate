struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct FSInput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uTime: f32;
@group(0) @binding(1) var uDiffuseTexture: texture_2d<f32>;
@group(0) @binding(2) var uDiffuseSampler: sampler;

@vertex
fn vs_main(vertex: Vertex) -> FSInput {

    var out: FSInput;
    out.position = vec4<f32>(vertex.position, 1.0);
    out.position.x += sin(uTime) * 0.5;
    out.uv = vertex.uv;
    return out;
}

@fragment
fn fs_main(in: FSInput) -> @location(0) vec4<f32> {

    var diffuse: vec4<f32> = textureSample(uDiffuseTexture, uDiffuseSampler, in.uv);
    
    return vec4<f32>(diffuse.xyz, 1.0);
}