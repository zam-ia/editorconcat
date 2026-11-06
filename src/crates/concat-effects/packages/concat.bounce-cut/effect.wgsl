struct Params { bounce: f32 }

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let rebound = sin(p * 3.14159265 * 2.0) * (1.0 - p) * params.bounce * 0.0015;
    let boundary = 1.0 - p + rebound;
    let feather = 0.008;
    let incoming = smoothstep(boundary - feather, boundary + feather, uv.y);
    let to_uv = vec2<f32>(uv.x, clamp(uv.y - boundary, 0.0, 1.0));
    var color = mix(from_at(uv), to_at(to_uv), incoming);
    let shadow = (1.0 - smoothstep(boundary - 0.035, boundary, uv.y)) *
        smoothstep(boundary - 0.08, boundary - 0.01, uv.y);
    color = vec4<f32>(color.rgb * (1.0 - shadow * 0.35), color.a);
    return color;
}
