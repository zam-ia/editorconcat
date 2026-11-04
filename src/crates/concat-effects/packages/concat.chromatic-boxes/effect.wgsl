struct Params { amount: f32 }

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let peak = sin(p * 3.14159265) * params.amount * 0.01;
    let grid = vec2<f32>(12.0, 18.0);
    let cell = floor(uv * grid);
    let n = hash(cell, floor(p * 30.0));
    let shift = (n - 0.5) * peak * 0.12;
    let q = clamp(vec2<f32>(uv.x + shift, uv.y), vec2<f32>(0.0), vec2<f32>(1.0));
    let channel = peak * 0.018;
    let a = vec4<f32>(
        from_at(clamp(q + vec2<f32>(channel, 0.0), vec2<f32>(0.0), vec2<f32>(1.0))).r,
        from_at(q).g,
        from_at(clamp(q - vec2<f32>(channel, 0.0), vec2<f32>(0.0), vec2<f32>(1.0))).b,
        from_at(q).a
    );
    let b = vec4<f32>(
        to_at(clamp(q - vec2<f32>(channel, 0.0), vec2<f32>(0.0), vec2<f32>(1.0))).r,
        to_at(q).g,
        to_at(clamp(q + vec2<f32>(channel, 0.0), vec2<f32>(0.0), vec2<f32>(1.0))).b,
        to_at(q).a
    );
    let delayed = smoothstep(n * 0.22, min(1.0, n * 0.22 + 0.62), p);
    return mix(a, b, delayed);
}
