struct Params {
    // 0 = S-Gamut3.Cine, 1 = S-Gamut3.
    gamut: f32,
}
// Sony's published 10-bit S-Log3 transfer, expressed on normalized code
// values. The branch below 171.2102946929/1023 keeps the linear toe.
fn slog3_to_linear(v: f32) -> f32 {
    let code = v * 1023.0;
    if (code >= 171.2102946929) {
        return (pow(10.0, (code - 420.0) / 261.5) * (0.18 + 0.01)) - 0.01;
    }
    return (code - 95.0) * 0.01125 / (171.2102946929 - 95.0);
}

fn effect(uv: vec2<f32>) -> vec4<f32> {
    let src = sample(uv);
    // SDR footage enters the shared shader chain decoded as display light;
    // restoring its encoded triplet here recovers the camera's S-Log3 code.
    let encoded = to_display(src.rgb);
    let linear = vec3<f32>(
        slog3_to_linear(encoded.r),
        slog3_to_linear(encoded.g),
        slog3_to_linear(encoded.b)
    );

    // S-Gamut3.Cine/S-Gamut3 -> XYZ D65 -> Rec.709 D65. Keeping the result
    // scene-linear lets primary corrections and creative looks run after the
    // technical transform without a second transfer conversion.
    let cine = mat3x3<f32>(
         1.62694741, -0.17851553, -0.04443611,
        -0.54013854,  1.41794093, -0.19591997,
        -0.08680887, -0.23942540,  1.24035608
    );
    let sgamut3 = mat3x3<f32>(
         1.87791513, -0.17680698, -0.02620113,
        -0.79416876,  1.35099962, -0.14842226,
        -0.08374637, -0.17419264,  1.17462339
    );
    let rec709 = select(cine * linear, sgamut3 * linear, params.gamut >= 0.5);
    return vec4<f32>(rec709, src.a);
}

