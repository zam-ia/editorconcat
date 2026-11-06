struct Params { heat: f32 }

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let centered = uv - vec2<f32>(0.5, 0.5);
    let grain = hash(floor(uv * vec2<f32>(48.0, 72.0)), 4.2) * 0.11;
    let distance = length(centered * vec2<f32>(0.86, 1.0)) + grain;
    let edge = progress * 0.82;
    let reveal = 1.0 - smoothstep(edge - 0.025, edge + 0.025, distance);
    var color = mix(from_at(uv), to_at(uv), reveal);
    let ring = (1.0 - smoothstep(0.018, 0.055, abs(distance - edge))) *
        sin(progress * 3.14159265);
    let heat = params.heat * 0.01;
    color = vec4<f32>(color.rgb + vec3<f32>(1.0, 0.28, 0.04) * ring * heat, color.a);
    return color;
}
