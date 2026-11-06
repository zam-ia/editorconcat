struct Params { tilt: f32 }

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let angle = p * params.tilt * 0.0028;
    let shifted = uv - vec2<f32>(p * 1.12, 0.0) - vec2<f32>(0.5, 0.5);
    let c = cos(angle);
    let s = sin(angle);
    let from_uv = vec2<f32>(shifted.x * c - shifted.y * s, shifted.x * s + shifted.y * c) +
        vec2<f32>(0.5, 0.5);
    let inside = step(0.0, from_uv.x) * step(from_uv.x, 1.0) *
        step(0.0, from_uv.y) * step(from_uv.y, 1.0);
    let card = from_at(clamp(from_uv, vec2<f32>(0.0), vec2<f32>(1.0)));
    let back = to_at(uv);
    let shadow = inside * (1.0 - smoothstep(0.0, 0.08, from_uv.x));
    return vec4<f32>(mix(back.rgb, card.rgb * (1.0 - shadow * 0.28), inside), mix(back.a, card.a, inside));
}
