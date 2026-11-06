struct Params { blades: f32 }

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let centered = uv - vec2<f32>(0.5, 0.5);
    let angle = atan2(centered.y, centered.x);
    let segment = 6.2831853 / max(3.0, params.blades);
    let polygon = length(centered) * cos(segment * 0.5) /
        max(0.001, cos(fract(angle / segment + 0.5) * segment - segment * 0.5));
    let edge = smoothstep(p * 0.78 - 0.025, p * 0.78 + 0.025, polygon);
    return mix(to_at(uv), from_at(uv), edge);
}
