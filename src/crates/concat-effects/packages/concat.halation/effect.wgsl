// Halation: the red-orange halo film lays around its blown highlights. Light
// passes through the emulsion, scatters off the backing and comes back, so
// the halo belongs to the picture's brightest parts and is drawn a little
// wider than they are - never a glow over the whole frame.
//
// The passes carry light, not colour: `across` and `down` each take the
// picture's own light, weighted above the threshold and premultiplied by
// alpha, and blur it. The last pass reads that bloom's luminance and lays
// the tint on with it, graded in the display encoding, where the halo of a
// film burn is judged and where 1.0 is white.
//
// A flat picture survives a normalised convolution unchanged, so on flat
// input `down_at` is `source_at` - which is what `fixtures.toml` pins.

// A WGSL package declares two things and nothing else: the `Params` struct
// whose fields are its knobs, in the manifest's order, and `fn effect`. The
// host reads the struct back through naga for each field's offset, so the
// order here is the layout there.
struct Params { threshold: f32, radius: f32, amount: f32, tint: f32 }

/// How far the taps reach, in sigmas. The Gaussian's width is `params.radius`
/// as a share of the layer, so a halo reads the same on a phone and a 4K cut.
const TAU: f32 = 3.14159265;
// Keep the blur loop statically bounded. D3D11's FXC compiler cannot compile
// implicit texture gradients inside the old per-pixel variable loop and could
// take the whole GPU device down while Concat generated effect previews.
// Thirty-two evenly spaced taps retain a smooth wide halo at a predictable
// cost on integrated GPUs.
const BLUR_TAPS: i32 = 32;

fn weight(d: f32, s: f32) -> f32 {
    return exp(-d * d / (2.0 * s * s));
}

/// The picture's own light where it is bright, premultiplied by alpha so a
/// clear pixel adds none, with colour kept: the halo takes its hue later.
fn source_at(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let d = to_display(max(c.rgb, vec3<f32>(0.0)));
    let level = luma(d) * max(c.a, 0.0);
    let th = params.threshold / 100.0;
    let passed = smoothstep(th, min(th + 0.25, 1.0), level);
    return vec4<f32>(c.rgb * passed * c.a, c.a);
}

/// `source_at`'s light weighed along its row about `uv`. The screen distance,
/// so the scaling trap either way is caught in the middle, where
/// `bloom_texel` is the picture's own pixel.
fn across(uv: vec2<f32>) -> vec4<f32> {
    let width = frame.size.x;
    if (width < 1.0) {
        return source_at(uv);
    }
    let screen = length((uv - vec2<f32>(0.5)) * vec2<f32>(2.0));
    let sigma = params.radius / 100.0 * screen * frame.size.y / TAU;
    if (sigma <= 0.01) {
        return source_at(uv);
    }
    var sum = vec4<f32>(0.0);
    var total = 0.0;
    for (var i = 0; i < BLUR_TAPS; i++) {
        let unit = (f32(i) + 0.5) / f32(BLUR_TAPS);
        let offset = (unit * 2.0 - 1.0) * TAU * sigma;
        let w = weight(offset, sigma);
        sum += source_at(vec2<f32>(uv.x + offset / width, uv.y)) * w;
        total += w;
    }
    if (total <= 0.0) {
        return source_at(uv);
    }
    return sum / total;
}

/// `across` weighed down its column about `uv`, in its own pixels.
fn down(uv: vec2<f32>) -> vec4<f32> {
    let rows = 1.0 / across_texel().y;
    let screen = length((uv - vec2<f32>(0.5)) * vec2<f32>(2.0));
    let sigma = params.radius / 100.0 * screen * 2.0;
    if (sigma <= 0.01) {
        return across_at(uv);
    }
    var sum = vec4<f32>(0.0);
    var total = 0.0;
    for (var i = 0; i < BLUR_TAPS; i++) {
        let unit = (f32(i) + 0.5) / f32(BLUR_TAPS);
        let offset = (unit * 2.0 - 1.0) * TAU * sigma;
        let w = weight(offset, sigma);
        sum += across_at(vec2<f32>(uv.x, uv.y + offset / rows)) * w;
        total += w;
    }
    if (total <= 0.0) {
        return across_at(uv);
    }
    return sum / total;
}

/// The halo's colour at a tint: negative the deep red of a dense negative,
/// positive the orange of a print, zero the red-orange of the stock.
fn halation_tint(t: f32) -> vec3<f32> {
    let s = t / 100.0;
    return vec3<f32>(1.0, 0.42 + 0.30 * s, 0.16 + 0.12 * s);
}

fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let base = to_display(c.rgb);
    let bloom = luma(down_at(uv).rgb);
    let lit = base + halation_tint(params.tint) * bloom * (params.amount / 100.0) * 1.6;
    return vec4<f32>(from_display(lit), c.a);
}
