// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Display-referred professional colour scopes.
//!
//! The input is the compositor's RGBA8 monitor result. It is therefore the
//! visible, processed frame (effects, grades, transitions and layers), not a
//! source file. RGBA8 monitor pixels are gamma-encoded Rec.709 display code
//! values. Luma is Y' = 0.2126 R' + 0.7152 G' + 0.0722 B'; vectorscope uses
//! Rec.709 Y'CbCr (Cb=(B'-Y')/1.8556, Cr=(R'-Y')/1.5748). No linear-light
//! value is mixed with those video-signal scales. HLG/PQ timelines are
//! explicitly labelled as their SDR tone-mapped monitor view.

use concat_project::model::ColorSpace;

pub const BINS: usize = 256;
const PLOT_W: usize = 384;
const PLOT_H: usize = 192;
const VECTOR_SIDE: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeSignal {
    Rec709Display,
    HlgToneMappedDisplay,
    PqToneMappedDisplay,
}

impl ScopeSignal {
    pub fn of(space: ColorSpace) -> Self {
        match space {
            ColorSpace::Sdr => Self::Rec709Display,
            ColorSpace::Hlg => Self::HlgToneMappedDisplay,
            ColorSpace::Pq => Self::PqToneMappedDisplay,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Rec709Display => "Rec.709 SDR",
            Self::HlgToneMappedDisplay => "HLG · vista SDR tone-mapped",
            Self::PqToneMappedDisplay => "PQ · vista SDR tone-mapped",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeBitmap {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct ScopeAnalysis {
    pub histogram: [[u32; BINS]; 4], // R, G, B, Y'
    pub waveform: Vec<u32>,          // 256 vertical levels per source column
    pub vectorscope: Vec<u32>,       // 256×256 Cb/Cr density
    pub source_width: u32,
    pub source_height: u32,
    pub clipped_low: [u64; 3],
    pub clipped_high: [u64; 3],
    pub signal: ScopeSignal,
    pub histogram_image: ScopeBitmap,
    pub waveform_image: ScopeBitmap,
    pub parade_image: ScopeBitmap,
    pub vectorscope_image: ScopeBitmap,
}

fn luma(r: u8, g: u8, b: u8) -> u8 {
    (0.2126 * f64::from(r) + 0.7152 * f64::from(g) + 0.0722 * f64::from(b))
        .round()
        .clamp(0.0, 255.0) as u8
}

fn blank(width: usize, height: usize) -> Vec<u8> {
    let mut out = vec![0; width * height * 4];
    for px in out.chunks_exact_mut(4) {
        px[0] = 12;
        px[1] = 14;
        px[2] = 18;
        px[3] = 255;
    }
    out
}

fn add(out: &mut [u8], width: usize, x: usize, y: usize, color: [u8; 3], strength: u8) {
    if x >= width || y * width * 4 + x * 4 + 3 >= out.len() {
        return;
    }
    let at = (y * width + x) * 4;
    for c in 0..3 {
        out[at + c] =
            out[at + c].saturating_add(((u16::from(color[c]) * u16::from(strength)) / 255) as u8);
    }
}

fn guides(out: &mut [u8], width: usize, height: usize) {
    for quarter in 1..4 {
        let y = quarter * height / 4;
        for x in 0..width {
            add(out, width, x, y, [28, 28, 32], 255);
        }
    }
}

fn density_strength(value: u32, max: u32) -> u8 {
    if value == 0 || max == 0 {
        0
    } else {
        ((f64::from(value).ln_1p() / f64::from(max).ln_1p()) * 235.0)
            .round()
            .clamp(18.0, 235.0) as u8
    }
}

fn histogram_bitmap(hist: &[[u32; BINS]; 4]) -> ScopeBitmap {
    let mut out = blank(PLOT_W, PLOT_H);
    guides(&mut out, PLOT_W, PLOT_H);
    let max = hist.iter().flatten().copied().max().unwrap_or(1);
    let colors = [
        [220, 45, 55],
        [40, 205, 90],
        [55, 105, 235],
        [185, 185, 185],
    ];
    for channel in 0..4 {
        for x in 0..PLOT_W {
            let bin = x * BINS / PLOT_W;
            let height =
                usize::from(density_strength(hist[channel][bin], max)) * (PLOT_H - 2) / 255;
            for y in PLOT_H.saturating_sub(height)..PLOT_H {
                add(
                    &mut out,
                    PLOT_W,
                    x,
                    y,
                    colors[channel],
                    if channel == 3 { 55 } else { 95 },
                );
            }
        }
    }
    ScopeBitmap {
        width: PLOT_W as u32,
        height: PLOT_H as u32,
        rgba: out,
    }
}

fn waveform_bitmap(density: &[u32], source_width: usize) -> ScopeBitmap {
    let mut out = blank(PLOT_W, PLOT_H);
    guides(&mut out, PLOT_W, PLOT_H);
    let max = density.iter().copied().max().unwrap_or(1);
    for x in 0..PLOT_W {
        let sx = (x * source_width / PLOT_W).min(source_width.saturating_sub(1));
        for level in 0..BINS {
            let value = density[sx * BINS + level];
            if value > 0 {
                let y = (BINS - 1 - level) * PLOT_H / BINS;
                add(
                    &mut out,
                    PLOT_W,
                    x,
                    y,
                    [105, 245, 160],
                    density_strength(value, max),
                );
            }
        }
    }
    ScopeBitmap {
        width: PLOT_W as u32,
        height: PLOT_H as u32,
        rgba: out,
    }
}

fn parade_bitmap(rgb_density: &[Vec<u32>; 3], source_width: usize) -> ScopeBitmap {
    let mut out = blank(PLOT_W, PLOT_H);
    guides(&mut out, PLOT_W, PLOT_H);
    let colors = [[245, 55, 65], [45, 235, 95], [70, 120, 255]];
    let third = PLOT_W / 3;
    for channel in 0..3 {
        let max = rgb_density[channel].iter().copied().max().unwrap_or(1);
        for local_x in 0..third {
            let sx = (local_x * source_width / third).min(source_width.saturating_sub(1));
            for level in 0..BINS {
                let value = rgb_density[channel][sx * BINS + level];
                if value > 0 {
                    let x = channel * third + local_x;
                    let y = (BINS - 1 - level) * PLOT_H / BINS;
                    add(
                        &mut out,
                        PLOT_W,
                        x,
                        y,
                        colors[channel],
                        density_strength(value, max),
                    );
                }
            }
        }
    }
    ScopeBitmap {
        width: PLOT_W as u32,
        height: PLOT_H as u32,
        rgba: out,
    }
}

fn vectorscope_bitmap(density: &[u32]) -> ScopeBitmap {
    let mut out = blank(VECTOR_SIDE, VECTOR_SIDE);
    let center = VECTOR_SIDE / 2;
    for i in 0..VECTOR_SIDE {
        add(&mut out, VECTOR_SIDE, center, i, [38, 38, 42], 255);
        add(&mut out, VECTOR_SIDE, i, center, [38, 38, 42], 255);
    }
    let radius = center as f64 * 0.9;
    for degree in 0..360 {
        let angle = f64::from(degree).to_radians();
        let x = (center as f64 + radius * angle.cos()).round() as usize;
        let y = (center as f64 - radius * angle.sin()).round() as usize;
        add(
            &mut out,
            VECTOR_SIDE,
            x.min(VECTOR_SIDE - 1),
            y.min(VECTOR_SIDE - 1),
            [34, 34, 40],
            255,
        );
    }
    let max = density.iter().copied().max().unwrap_or(1);
    for y in 0..VECTOR_SIDE {
        for x in 0..VECTOR_SIDE {
            let value = density[y * VECTOR_SIDE + x];
            if value > 0 {
                add(
                    &mut out,
                    VECTOR_SIDE,
                    x,
                    y,
                    [90, 225, 220],
                    density_strength(value, max),
                );
            }
        }
    }
    ScopeBitmap {
        width: VECTOR_SIDE as u32,
        height: VECTOR_SIDE as u32,
        rgba: out,
    }
}

/// Analyse one processed RGBA8 monitor frame and render all four instruments.
pub fn analyse_rgba(
    rgba: &[u8],
    width: u32,
    height: u32,
    signal: ScopeSignal,
) -> Result<ScopeAnalysis, String> {
    let expected = width as usize * height as usize * 4;
    if width == 0 || height == 0 || rgba.len() != expected {
        return Err(format!(
            "scope frame is {} bytes, expected {expected}",
            rgba.len()
        ));
    }
    let w = width as usize;
    let mut histogram = [[0u32; BINS]; 4];
    let mut waveform = vec![0u32; w * BINS];
    let mut rgb_density = [
        vec![0u32; w * BINS],
        vec![0u32; w * BINS],
        vec![0u32; w * BINS],
    ];
    let mut vectorscope = vec![0u32; VECTOR_SIDE * VECTOR_SIDE];
    let mut clipped_low = [0u64; 3];
    let mut clipped_high = [0u64; 3];

    for (index, px) in rgba.chunks_exact(4).enumerate() {
        if px[3] == 0 {
            continue;
        }
        let x = index % w;
        let [r, g, b] = [px[0], px[1], px[2]];
        let y = luma(r, g, b);
        for (channel, value) in [r, g, b].into_iter().enumerate() {
            histogram[channel][value as usize] += 1;
            rgb_density[channel][x * BINS + value as usize] += 1;
            clipped_low[channel] += u64::from(value <= 1);
            clipped_high[channel] += u64::from(value >= 254);
        }
        histogram[3][y as usize] += 1;
        waveform[x * BINS + y as usize] += 1;

        let rf = f64::from(r) / 255.0;
        let bf = f64::from(b) / 255.0;
        let yf = f64::from(y) / 255.0;
        let cb = (bf - yf) / 1.8556;
        let cr = (rf - yf) / 1.5748;
        let vx = ((cb + 0.5) * 255.0).round().clamp(0.0, 255.0) as usize;
        let vy = ((0.5 - cr) * 255.0).round().clamp(0.0, 255.0) as usize;
        vectorscope[vy * VECTOR_SIDE + vx] += 1;
    }

    Ok(ScopeAnalysis {
        histogram_image: histogram_bitmap(&histogram),
        waveform_image: waveform_bitmap(&waveform, w),
        parade_image: parade_bitmap(&rgb_density, w),
        vectorscope_image: vectorscope_bitmap(&vectorscope),
        histogram,
        waveform,
        vectorscope,
        source_width: width,
        source_height: height,
        clipped_low,
        clipped_high,
        signal,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(width: u32, height: u32, rgb: [u8; 3]) -> Vec<u8> {
        [rgb[0], rgb[1], rgb[2], 255]
            .into_iter()
            .cycle()
            .take(width as usize * height as usize * 4)
            .collect()
    }

    #[test]
    fn black_white_and_neutral_grey_land_in_exact_signal_bins() {
        for (rgb, bin) in [
            ([0, 0, 0], 0),
            ([128, 128, 128], 128),
            ([255, 255, 255], 255),
        ] {
            let result = analyse_rgba(&solid(4, 3, rgb), 4, 3, ScopeSignal::Rec709Display).unwrap();
            for channel in 0..4 {
                assert_eq!(result.histogram[channel][bin], 12);
            }
        }
    }

    #[test]
    fn horizontal_gradient_keeps_position_in_the_waveform() {
        let mut pixels = Vec::new();
        for value in [0u8, 85, 170, 255] {
            pixels.extend_from_slice(&[value, value, value, 255]);
        }
        let result = analyse_rgba(&pixels, 4, 1, ScopeSignal::Rec709Display).unwrap();
        for (x, level) in [0usize, 85, 170, 255].into_iter().enumerate() {
            assert_eq!(result.waveform[x * BINS + level], 1);
        }
    }

    #[test]
    fn rgb_bars_separate_channels_and_report_clipping() {
        let pixels = [255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255];
        let result = analyse_rgba(&pixels, 3, 1, ScopeSignal::Rec709Display).unwrap();
        assert_eq!(result.histogram[0][255], 1);
        assert_eq!(result.histogram[1][255], 1);
        assert_eq!(result.histogram[2][255], 1);
        assert_eq!(result.clipped_high, [1, 1, 1]);
        assert_eq!(result.clipped_low, [2, 2, 2]);
    }

    #[test]
    fn vectorscope_neutral_is_centre_and_primaries_are_not() {
        let neutral = analyse_rgba(
            &solid(1, 1, [128, 128, 128]),
            1,
            1,
            ScopeSignal::Rec709Display,
        )
        .unwrap();
        assert_eq!(neutral.vectorscope[128 * VECTOR_SIDE + 128], 1);
        let red =
            analyse_rgba(&solid(1, 1, [255, 0, 0]), 1, 1, ScopeSignal::Rec709Display).unwrap();
        assert_eq!(red.vectorscope[128 * VECTOR_SIDE + 128], 0);
        assert_eq!(red.vectorscope.iter().sum::<u32>(), 1);
    }

    #[test]
    fn scopes_do_not_mutate_the_frame_and_render_compact_results() {
        let pixels = solid(320, 180, [32, 96, 192]);
        let before = pixels.clone();
        let started = std::time::Instant::now();
        let result = analyse_rgba(&pixels, 320, 180, ScopeSignal::HlgToneMappedDisplay).unwrap();
        assert_eq!(pixels, before);
        assert_eq!(result.histogram_image.rgba.len(), PLOT_W * PLOT_H * 4);
        assert!(started.elapsed() < std::time::Duration::from_secs(2));
    }
}
