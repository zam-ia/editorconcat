struct Params { tilt: f32 }

fn tilted(uv: vec2<f32>, angle: f32, shift: vec2<f32>, zoom: f32) -> vec2<f32> {
    let q = (uv - vec2<f32>(0.5, 0.5) - shift) / zoom;
    let c = cos(angle);
    let s = sin(angle);
    return clamp(vec2<f32>(q.x * c - q.y * s, q.x * s + q.y * c) + vec2<f32>(0.5, 0.5), vec2<f32>(0.0), vec2<f32>(1.0));
}

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let peak = sin(p * 3.14159265);
    let angle = peak * params.tilt * 0.0018;
    let a_uv = tilted(uv, angle, vec2<f32>(p * -0.16, peak * 0.025), 1.0 + peak * 0.12);
    let b_uv = tilted(uv, -angle, vec2<f32>((1.0 - p) * 0.16, -peak * 0.025), 1.0 + peak * 0.12);
    return mix(from_at(a_uv), to_at(b_uv), p);
}
