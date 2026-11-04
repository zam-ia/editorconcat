struct Params { brightness: f32 }

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let axis = uv.x + uv.y * 0.18;
    let edge = p * 1.18;
    let reveal = 1.0 - smoothstep(edge - 0.025, edge + 0.025, axis);
    var color = mix(from_at(uv), to_at(uv), reveal);
    let beam = exp(-pow(axis - edge, 2.0) * 520.0);
    let halo = exp(-pow(axis - edge, 2.0) * 55.0);
    let k = params.brightness * 0.01;
    color = vec4<f32>(color.rgb + vec3<f32>(1.0, 0.93, 0.78) * beam * k + vec3<f32>(0.35, 0.55, 1.0) * halo * k * 0.18, color.a);
    return color;
}
