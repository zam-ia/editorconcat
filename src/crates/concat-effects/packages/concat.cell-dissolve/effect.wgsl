struct Params { cells: f32 }

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let grid = vec2<f32>(max(4.0, params.cells), max(4.0, params.cells) * 1.7);
    let cell = floor(uv * grid);
    let local = fract(uv * grid) - vec2<f32>(0.5, 0.5);
    let noise = hash(cell, 9.17);
    let radius = smoothstep(noise * 0.72, noise * 0.72 + 0.28, progress) * 0.9;
    let reveal = 1.0 - smoothstep(radius - 0.14, radius, length(local));
    return mix(from_at(uv), to_at(uv), reveal);
}
