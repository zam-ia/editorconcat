struct Params { glow: f32 }

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let peak = pow(sin(p * 3.14159265), 2.0);
    let spread = 0.004 + peak * 0.016;
    var soft = vec4<f32>(0.0);
    soft += mix(from_at(uv + vec2<f32>(spread, 0.0)), to_at(uv + vec2<f32>(spread, 0.0)), p);
    soft += mix(from_at(uv - vec2<f32>(spread, 0.0)), to_at(uv - vec2<f32>(spread, 0.0)), p);
    soft += mix(from_at(uv + vec2<f32>(0.0, spread)), to_at(uv + vec2<f32>(0.0, spread)), p);
    soft += mix(from_at(uv - vec2<f32>(0.0, spread)), to_at(uv - vec2<f32>(0.0, spread)), p);
    soft = soft * 0.25;
    let base = mix(from_at(uv), to_at(uv), p);
    let k = peak * params.glow * 0.01;
    return vec4<f32>(mix(base.rgb, min(vec3<f32>(1.0), soft.rgb + vec3<f32>(0.65)), k * 0.82), base.a);
}
