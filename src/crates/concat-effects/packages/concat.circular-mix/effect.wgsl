struct Params { softness: f32 }

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    if (progress <= 0.001) { return from_at(uv); }
    if (progress >= 0.999) { return to_at(uv); }
    let d = distance(uv, vec2<f32>(0.5, 0.5));
    let radius = progress * 0.82;
    let feather = 0.002 + params.softness * 0.00045;
    let reveal = 1.0 - smoothstep(radius - feather, radius + feather, d);
    return mix(from_at(uv), to_at(uv), reveal);
}
