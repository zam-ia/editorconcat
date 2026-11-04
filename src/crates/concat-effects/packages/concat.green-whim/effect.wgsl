struct Params { energy: f32 }

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let peak = sin(p * 3.14159265);
    let k = params.energy * 0.0009 * peak;
    let wave = vec2<f32>(
        sin(uv.y * 25.0 + p * 14.0),
        cos(uv.x * 19.0 - p * 11.0)
    ) * k;
    let a = from_at(clamp(uv + wave, vec2<f32>(0.0), vec2<f32>(1.0)));
    let b = to_at(clamp(uv - wave, vec2<f32>(0.0), vec2<f32>(1.0)));
    var color = mix(a, b, p);
    let ribbon = exp(-pow(uv.x + sin(uv.y * 8.0 + p * 9.0) * 0.08 - p, 2.0) * 80.0);
    color = vec4<f32>(color.rgb + vec3<f32>(0.08, 0.5, 0.2) * ribbon * peak * params.energy * 0.012, color.a);
    return color;
}
