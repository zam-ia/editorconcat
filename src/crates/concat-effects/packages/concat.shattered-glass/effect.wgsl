struct Params { scatter: f32 }

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let cells = vec2<f32>(9.0, 13.0);
    let cell = floor(uv * cells);
    let local = fract(uv * cells);
    let n = hash(cell, 2.7);
    let angle = n * 6.2831853;
    let direction = vec2<f32>(cos(angle), sin(angle));
    let burst = sin(p * 3.14159265) * params.scatter * 0.0011;
    let displaced = clamp(uv + direction * burst * (0.45 + n), vec2<f32>(0.0), vec2<f32>(1.0));
    let shard_delay = n * 0.32;
    let reveal = smoothstep(shard_delay, min(1.0, shard_delay + 0.5), p);
    var color = mix(from_at(displaced), to_at(uv), reveal);
    let border = min(min(local.x, 1.0 - local.x), min(local.y, 1.0 - local.y));
    let crack = (1.0 - smoothstep(0.0, 0.025, border)) * sin(p * 3.14159265);
    color = vec4<f32>(color.rgb + vec3<f32>(crack * 0.32), color.a);
    return color;
}
