struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct FSInput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@group(0) @binding(0) var<uniform> uTime: f32;

@vertex
fn vs_main(vertex: Vertex) -> FSInput {

    var out: FSInput;
    out.position = vec4<f32>(vertex.position, 1.0);
    out.position.x += sin(uTime) * 0.5;
    out.color = vertex.color;
    return out;
}

@fragment
fn fs_main(in: FSInput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}