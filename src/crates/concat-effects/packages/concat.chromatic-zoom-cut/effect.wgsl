struct Params { fringe: f32 }

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let pulse = sin(p * 3.14159265);
    let centered = uv - vec2<f32>(0.5, 0.5);
    let zoomed = centered / (1.0 + pulse * 0.32) + vec2<f32>(0.5, 0.5);
    let split = normalize(centered + vec2<f32>(0.0001, 0.0001)) *
        pulse * params.fringe * 0.00016;
    let base = mix(from_at(zoomed), to_at(zoomed), p);
    let red = mix(from_at(zoomed + split), to_at(zoomed + split), p).r;
    let blue = mix(from_at(zoomed - split), to_at(zoomed - split), p).b;
    return vec4<f32>(red, base.g, blue, base.a);
}
