// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! EBU R128 / ITU-R BS.1770 loudness analysis over decoded audio.
//!
//! The linked FFmpeg already carries the standards-tested `ebur128` filter,
//! including its true-peak oversampler.  Reading its frame metadata keeps the
//! measurement in the same libavfilter build as the editor's mix and avoids a
//! second native DSP dependency.  `astats` is deliberately not used for true
//! peak: a sampled maximum is not an inter-sample peak.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::error::{Error, Result};
use crate::samples::{AudioDecoder, AudioOptions, SampleFormat};

/// One completed programme or clip measurement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoudnessMeasurement {
    /// Gated programme loudness, LUFS. `None` for digital silence.
    pub integrated_lufs: Option<f64>,
    /// Loudness of the last 400 ms window, LUFS.
    pub momentary_lufs: Option<f64>,
    /// Highest 400 ms loudness seen in the programme, LUFS.
    pub max_momentary_lufs: Option<f64>,
    /// Loudness of the last three-second window, LUFS.
    pub short_term_lufs: Option<f64>,
    /// Highest three-second loudness seen in the programme, LUFS.
    pub max_short_term_lufs: Option<f64>,
    /// Loudness range, LU.
    pub loudness_range_lu: Option<f64>,
    /// Oversampled programme maximum, dBTP.
    pub true_peak_dbtp: Option<f64>,
    /// Maximum decoded sample, dBFS. Kept distinct from true peak.
    pub sample_peak_dbfs: Option<f64>,
    /// Samples at or beyond full scale in the analysed output.
    pub clipped_samples: u64,
    /// Analysed duration, from the decoded sample count.
    pub duration_seconds: f64,
    /// The analysis output rate. The R128 filter itself sees the source
    /// layout/rate before this standardised sink conversion.
    pub output_sample_rate: u32,
    /// The analysis output channel count.
    pub output_channels: u16,
}

impl LoudnessMeasurement {
    /// True when at least one decoded sample reached full scale.
    pub fn clipped(self) -> bool {
        self.clipped_samples > 0
    }
}

/// Editable loudness target. The UI starts at -16 LUFS / -1 dBTP, but these
/// are ordinary values rather than a universal preset baked into processing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NormalizationTarget {
    /// Desired integrated programme loudness.
    pub integrated_lufs: f64,
    /// Maximum reconstructed (inter-sample) peak.
    pub max_true_peak_dbtp: f64,
}

impl Default for NormalizationTarget {
    fn default() -> Self {
        Self {
            integrated_lufs: -16.0,
            max_true_peak_dbtp: -1.0,
        }
    }
}

/// Processing policy chosen by the user. Both modes are transparent gain
/// operations over the already-rendered signal; neither rewrites a source.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NormalizationMode {
    /// Pursue the LUFS target, constrained by the true-peak ceiling.
    #[default]
    LoudnessAndPeak,
    /// Never amplify: useful when delivery policy only permits attenuation.
    AttenuateOnly,
}

/// A constant-gain normalisation decision. Applying one gain to every
/// audible branch commutes with fades and the final sum, so a full mix and a
/// single clip use the same plan without baking or changing source files.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NormalizationPlan {
    /// Gain that would hit the loudness target without constraints.
    pub requested_gain_db: f64,
    /// Gain safe to apply after the peak and range constraints.
    pub applied_gain_db: f64,
    /// Integrated loudness expected from constant gain.
    pub predicted_integrated_lufs: f64,
    /// True peak expected from constant gain.
    pub predicted_true_peak_dbtp: Option<f64>,
    /// False when the true-peak ceiling (or the editor's +24 dB safety
    /// ceiling) prevents the requested loudness target.
    pub target_reachable: bool,
    /// True when the dBTP ceiling, rather than loudness, set the gain.
    pub limited_by_true_peak: bool,
    /// True when the editor's finite gain range set the gain.
    pub limited_by_gain_range: bool,
    /// True when the attenuate-only policy rejected requested amplification.
    pub limited_by_mode: bool,
}

impl NormalizationPlan {
    /// Plans transparent constant-gain normalisation. Returns `None` for
    /// silence, whose integrated loudness is undefined.
    pub fn from_measurement(
        measurement: LoudnessMeasurement,
        target: NormalizationTarget,
    ) -> Option<Self> {
        Self::from_measurement_with_mode(measurement, target, NormalizationMode::LoudnessAndPeak)
    }

    /// Plans normalization with an explicit, implemented processing policy.
    pub fn from_measurement_with_mode(
        measurement: LoudnessMeasurement,
        target: NormalizationTarget,
        mode: NormalizationMode,
    ) -> Option<Self> {
        let integrated = measurement.integrated_lufs?;
        let requested = target.integrated_lufs - integrated;
        let peak_limit = measurement
            .true_peak_dbtp
            .map_or(f64::INFINITY, |peak| target.max_true_peak_dbtp - peak);
        // Keep the operation within the inspector's existing gain range.
        // Attenuation may go to silence; amplification stops at +24 dB.
        let mode_limit = match mode {
            NormalizationMode::LoudnessAndPeak => f64::INFINITY,
            NormalizationMode::AttenuateOnly => 0.0,
        };
        let applied = requested
            .min(peak_limit)
            .min(mode_limit)
            .min(24.0)
            .max(-120.0);
        let limited_by_true_peak = peak_limit + 0.01 < requested;
        let limited_by_gain_range = requested > 24.0 + 0.01 || requested < -120.0 - 0.01;
        let limited_by_mode = mode_limit + 0.01 < requested;
        Some(Self {
            requested_gain_db: requested,
            applied_gain_db: applied,
            predicted_integrated_lufs: integrated + applied,
            predicted_true_peak_dbtp: measurement.true_peak_dbtp.map(|peak| peak + applied),
            target_reachable: !limited_by_true_peak && !limited_by_gain_range && !limited_by_mode,
            limited_by_true_peak,
            limited_by_gain_range,
            limited_by_mode,
        })
    }

    /// Linear multiplier persisted by the editor's existing volume command.
    pub fn linear_gain(self) -> f64 {
        10.0f64.powf(self.applied_gain_db / 20.0)
    }
}

fn finite_loudness(value: f64) -> Option<f64> {
    // FFmpeg uses about -120.7 for an empty R128 window. Surface that as
    // silence rather than a plausible programme measurement.
    // -70 LUFS is the absolute gate in BS.1770. FFmpeg reports that edge
    // for silence in some versions and -120.7 in others.
    (value.is_finite() && value > -70.0).then_some(value)
}

fn db_of(linear: f64) -> Option<f64> {
    (linear.is_finite() && linear > 0.0).then(|| 20.0 * linear.log10())
}

fn number(metadata: &ffmpeg_the_third::DictionaryRef<'_>, key: &str) -> Option<f64> {
    metadata
        .into_iter()
        .find_map(|(held, value)| (held == key).then(|| value.parse::<f64>().ok()).flatten())
}

/// Analyses one file/stream through FFmpeg's R128 filter. `progress` is
/// monotonic in 0..=1 and is called from the worker thread.
pub fn analyse_file(
    path: impl AsRef<Path>,
    stream: Option<usize>,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(f32),
) -> Result<LoudnessMeasurement> {
    if cancel.load(Ordering::Relaxed) {
        return Err(Error::Cancelled);
    }
    let path = path.as_ref();
    let info = crate::probe(path).ok();
    let expected_seconds = info
        .as_ref()
        .and_then(|info| info.duration)
        .map(|duration| duration.as_f64())
        .filter(|seconds| *seconds > 0.0);
    let audio = info.as_ref().and_then(|info| {
        stream
            .and_then(|wanted| {
                info.audio_streams
                    .iter()
                    .find(|audio| audio.index as usize == wanted)
            })
            .or(info.audio.as_ref())
    });
    let output_rate = audio
        .map(|audio| audio.sample_rate)
        .filter(|rate| *rate > 0)
        .unwrap_or(48_000);
    let output_channels = audio
        .map(|audio| audio.channels as u16)
        .filter(|channels| matches!(channels, 1 | 2))
        .unwrap_or(2);
    let options = AudioOptions {
        stream,
        rate: output_rate,
        channels: output_channels,
        format: SampleFormat::F32,
        // peak=true is FFmpeg's oversampled true-peak path. metadata=1
        // exposes M/S/I/LRA and the linear true peak on each 100 ms frame.
        filters: vec!["ebur128=metadata=1:peak=true".to_owned()],
        ..AudioOptions::default()
    };
    let mut decoder = AudioDecoder::open(path, &options)?;
    let mut integrated = None;
    let mut momentary = None;
    let mut short_term = None;
    let mut max_momentary: Option<f64> = None;
    let mut max_short_term: Option<f64> = None;
    let mut lra = None;
    let mut true_peak_linear: f64 = 0.0;
    let mut sample_peak: f64 = 0.0;
    let mut clipped_samples = 0u64;
    let mut frames_per_channel = 0u64;
    let mut last_report = -1.0f32;

    while let Some(frame) = decoder.next_frame()? {
        if cancel.load(Ordering::Relaxed) {
            return Err(Error::Cancelled);
        }
        let metadata = frame.metadata();
        if let Some(value) = number(&metadata, "lavfi.r128.I").and_then(finite_loudness) {
            integrated = Some(value);
        }
        if let Some(value) = number(&metadata, "lavfi.r128.M").and_then(finite_loudness) {
            momentary = Some(value);
            max_momentary = Some(max_momentary.map_or(value, |held| held.max(value)));
        }
        if let Some(value) = number(&metadata, "lavfi.r128.S").and_then(finite_loudness) {
            short_term = Some(value);
            max_short_term = Some(max_short_term.map_or(value, |held| held.max(value)));
        }
        if let Some(value) = number(&metadata, "lavfi.r128.LRA")
            && value.is_finite()
        {
            lra = Some(value.max(0.0));
        }
        if let Some(value) = number(&metadata, "lavfi.r128.true_peak")
            && value.is_finite()
        {
            true_peak_linear = true_peak_linear.max(value.abs());
        }

        let bytes = decoder.bytes_of(&frame);
        for chunk in bytes.chunks_exact(4) {
            let sample = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) as f64;
            if sample.is_finite() {
                let magnitude = sample.abs();
                sample_peak = sample_peak.max(magnitude);
                if magnitude >= 1.0 {
                    clipped_samples += 1;
                }
            }
        }
        frames_per_channel += frame.samples() as u64;
        let seconds = frames_per_channel as f64 / f64::from(options.rate);
        let fraction =
            expected_seconds.map_or(0.0, |total| (seconds / total).clamp(0.0, 0.99)) as f32;
        if fraction - last_report >= 0.01 {
            last_report = fraction;
            progress(fraction);
        }
    }
    progress(1.0);

    Ok(LoudnessMeasurement {
        integrated_lufs: integrated,
        momentary_lufs: momentary,
        max_momentary_lufs: max_momentary,
        short_term_lufs: short_term,
        max_short_term_lufs: max_short_term,
        loudness_range_lu: lra,
        true_peak_dbtp: db_of(true_peak_linear),
        sample_peak_dbfs: db_of(sample_peak),
        clipped_samples,
        duration_seconds: frames_per_channel as f64 / f64::from(options.rate),
        output_sample_rate: options.rate,
        output_channels: options.channels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    fn wav(path: &Path, rate: u32, channels: u16, seconds: f64, sample: impl Fn(u64, u16) -> f32) {
        let frames = (seconds * f64::from(rate)).round() as u64;
        let data_len = frames * u64::from(channels) * 4;
        let mut bytes = Vec::with_capacity(44 + data_len as usize);
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + data_len as u32).to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&3u16.to_le_bytes()); // IEEE float
        bytes.extend_from_slice(&channels.to_le_bytes());
        bytes.extend_from_slice(&rate.to_le_bytes());
        bytes.extend_from_slice(&(rate * u32::from(channels) * 4).to_le_bytes());
        bytes.extend_from_slice(&(channels * 4).to_le_bytes());
        bytes.extend_from_slice(&32u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&(data_len as u32).to_le_bytes());
        for frame in 0..frames {
            for channel in 0..channels {
                bytes.extend_from_slice(&sample(frame, channel).to_le_bytes());
            }
        }
        std::fs::write(path, bytes).expect("writes reference wav");
    }

    fn measure(
        name: &str,
        rate: u32,
        channels: u16,
        sample: impl Fn(u64, u16) -> f32,
    ) -> LoudnessMeasurement {
        let path =
            std::env::temp_dir().join(format!("concat-meter-{name}-{}.wav", std::process::id()));
        wav(&path, rate, channels, 4.0, sample);
        let result =
            analyse_file(&path, None, &AtomicBool::new(false), &mut |_| {}).expect("analyses");
        let _ = std::fs::remove_file(path);
        result
    }

    #[test]
    fn silence_has_no_fictional_loudness_or_peak() {
        let result = measure("silence", 48_000, 2, |_, _| 0.0);
        assert_eq!(result.integrated_lufs, None);
        assert_eq!(result.true_peak_dbtp, None);
        assert_eq!(result.sample_peak_dbfs, None);
        assert!(!result.clipped());
    }

    #[test]
    fn a_mono_reference_tone_is_measured_in_lufs() {
        let result = measure("mono-tone", 48_000, 1, |frame, _| {
            (0.1 * (2.0 * PI * 1_000.0 * frame as f64 / 48_000.0).sin()) as f32
        });
        let integrated = result.integrated_lufs.expect("tone is measurable");
        assert!(
            (integrated - -23.0).abs() < 0.4,
            "measured {integrated} LUFS"
        );
        let peak = result.sample_peak_dbfs.unwrap();
        assert!((peak - -20.0).abs() < 0.1, "sample peak was {peak:.3} dBFS");
    }

    #[test]
    fn stereo_and_another_sample_rate_keep_the_r128_channel_weighting() {
        let result = measure("stereo-44100", 44_100, 2, |frame, _| {
            (0.1 * (2.0 * PI * 1_000.0 * frame as f64 / 44_100.0).sin()) as f32
        });
        let integrated = result.integrated_lufs.expect("tone is measurable");
        assert!(
            (integrated - -20.0).abs() < 0.5,
            "measured {integrated} LUFS"
        );
    }

    #[test]
    fn inter_sample_peak_is_not_reported_as_sample_peak() {
        // A quarter-rate sine, shifted halfway between its maxima: samples
        // peak at sqrt(1/2), while the reconstructed waveform peaks near 1.
        let result = measure("intersample", 48_000, 1, |frame, _| {
            (0.9 * (2.0 * PI * 12_000.0 * frame as f64 / 48_000.0 + PI / 4.0).sin()) as f32
        });
        let sample = result.sample_peak_dbfs.expect("sample peak");
        let true_peak = result.true_peak_dbtp.expect("true peak");
        assert!(
            true_peak > sample + 2.0,
            "sample {sample:.2} dBFS, true {true_peak:.2} dBTP"
        );
    }

    #[test]
    fn cancellation_is_observed_before_decode() {
        let cancelled = AtomicBool::new(true);
        let result = analyse_file("does-not-need-to-exist.wav", None, &cancelled, &mut |_| {});
        assert!(matches!(result, Err(Error::Cancelled)));
    }

    #[test]
    fn normalisation_reports_a_true_peak_limited_target() {
        let measurement = LoudnessMeasurement {
            integrated_lufs: Some(-24.0),
            true_peak_dbtp: Some(-3.0),
            momentary_lufs: None,
            max_momentary_lufs: None,
            short_term_lufs: None,
            max_short_term_lufs: None,
            loudness_range_lu: None,
            sample_peak_dbfs: Some(-4.0),
            clipped_samples: 0,
            duration_seconds: 1.0,
            output_sample_rate: 48_000,
            output_channels: 2,
        };
        let plan = NormalizationPlan::from_measurement(measurement, NormalizationTarget::default())
            .unwrap();
        assert_eq!(plan.requested_gain_db, 8.0);
        assert_eq!(plan.applied_gain_db, 2.0);
        assert!(plan.limited_by_true_peak);
        assert!(!plan.target_reachable);
        assert_eq!(plan.predicted_true_peak_dbtp, Some(-1.0));
    }

    #[test]
    fn attenuate_only_mode_never_amplifies() {
        let measurement = LoudnessMeasurement {
            integrated_lufs: Some(-20.0),
            true_peak_dbtp: Some(-8.0),
            momentary_lufs: None,
            max_momentary_lufs: None,
            short_term_lufs: None,
            max_short_term_lufs: None,
            loudness_range_lu: None,
            sample_peak_dbfs: Some(-9.0),
            clipped_samples: 0,
            duration_seconds: 1.0,
            output_sample_rate: 48_000,
            output_channels: 2,
        };
        let plan = NormalizationPlan::from_measurement_with_mode(
            measurement,
            NormalizationTarget::default(),
            NormalizationMode::AttenuateOnly,
        )
        .unwrap();
        assert_eq!(plan.requested_gain_db, 4.0);
        assert_eq!(plan.applied_gain_db, 0.0);
        assert!(plan.limited_by_mode);
        assert!(!plan.target_reachable);
    }

    /// P1 audiovisual contract: analyse the encoded output of the same mixer
    /// export uses, apply the plan to two real branches, render again, then
    /// verify the file rather than trusting the predicted number. The fixture
    /// covers multipista, volume, fades and an FFmpeg filter chain.
    #[test]
    fn rendered_mix_normalization_matches_the_export_signal() {
        use crate::audio::{AudioClip, mix_to_file};
        use concat_core::animate::Track;

        let stem = std::env::temp_dir().join(format!(
            "concat-meter-export-fixture-{}",
            std::process::id()
        ));
        let source = stem.with_extension("wav");
        let before_path = stem.with_extension("before.m4a");
        let after_path = stem.with_extension("after.m4a");
        wav(&source, 48_000, 2, 4.0, |frame, channel| {
            let frequency = if channel == 0 { 440.0 } else { 660.0 };
            (0.08 * (2.0 * PI * frequency * frame as f64 / 48_000.0).sin()) as f32
        });
        let mut clips = vec![
            AudioClip {
                path: source.clone(),
                stream: None,
                start: 0.0,
                duration: 4.0,
                source_start: 0.0,
                speed: 1.0,
                preserve_pitch: true,
                volume: 0.5,
                volume_curve: Track::default(),
                fade_in: 0.4,
                fade_out: 0.0,
                filter_chain: "highpass=f=80".to_owned(),
            },
            AudioClip {
                path: source.clone(),
                stream: None,
                start: 1.0,
                duration: 3.0,
                source_start: 0.5,
                speed: 1.0,
                preserve_pitch: true,
                volume: 0.25,
                volume_curve: Track::default(),
                fade_in: 0.0,
                fade_out: 0.5,
                filter_chain: String::new(),
            },
        ];
        mix_to_file(&clips, 4.0, &before_path).expect("renders pre-normalization mix");
        let before = analyse_file(&before_path, None, &AtomicBool::new(false), &mut |_| {})
            .expect("measures pre-normalization mix");
        let target = NormalizationTarget::default();
        let plan = NormalizationPlan::from_measurement(before, target).expect("non-silent mix");
        let gain = plan.linear_gain();
        for clip in &mut clips {
            clip.volume *= gain;
        }
        mix_to_file(&clips, 4.0, &after_path).expect("renders post-normalization mix");
        let after = analyse_file(&after_path, None, &AtomicBool::new(false), &mut |_| {})
            .expect("measures post-normalization mix");
        let before_i = before.integrated_lufs.expect("before LUFS");
        let after_i = after.integrated_lufs.expect("after LUFS");
        let after_tp = after.true_peak_dbtp.expect("after true peak");
        eprintln!(
            "P1 reference mix: before={before_i:.2} LUFS, after={after_i:.2} LUFS, after_tp={after_tp:.2} dBTP"
        );
        assert!(
            (after_i - target.integrated_lufs).abs() <= 0.5,
            "post export was {after_i:.2} LUFS"
        );
        assert!(
            after_tp <= target.max_true_peak_dbtp + 0.1,
            "post export was {after_tp:.2} dBTP"
        );
        for path in [source, before_path, after_path] {
            let _ = std::fs::remove_file(path);
        }
    }
}
