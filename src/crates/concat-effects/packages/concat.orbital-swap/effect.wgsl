struct Params { turn: f32 }

fn orbit_uv(uv: vec2<f32>, angle: f32, scale: f32, shift: vec2<f32>) -> vec2<f32> {
    let p = (uv - vec2<f32>(0.5, 0.5) - shift) / scale;
    let c = cos(angle);
    let s = sin(angle);
    return vec2<f32>(p.x * c - p.y * s, p.x * s + p.y * c) + vec2<f32>(0.5, 0.5);
}

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let peak = sin(p * 3.14159265);
    let angle = peak * (0.25 + params.turn * 0.0065);
    let radius = peak * 0.16;
    let a_uv = orbit_uv(uv, angle, 1.0 + peak * 0.18, vec2<f32>(-radius, 0.0));
    let b_uv = orbit_uv(uv, -angle, 1.0 + peak * 0.18, vec2<f32>(radius, 0.0));
    return mix(from_at(a_uv), to_at(b_uv), p);
}
