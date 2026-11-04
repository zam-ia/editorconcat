// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! A manual, on-disk preview render for the timeline.
//!
//! This is deliberately never automatic. A render is a relatively expensive
//! export at monitor resolution; the editor only starts one from the
//! timeline's context menu. While its document revision still matches, the
//! monitor reads the baked file instead of compositing every source clip and
//! effect again. Audio continues to come from the live timeline, so scrubbing,
//! meters and track mute remain responsive.

use std::path::PathBuf;
use std::sync::Arc;

use concat_export::{ClipKind, ExportClip};
use concat_host::export::{self, ColorRange, ExportSpec, VideoCodec};

use super::Studio;
use crate::host::spawn_in_project;
use crate::i18n::t;

/// One usable render cache. The file contains the whole timeline; `from` and
/// `to` say which part the user asked the monitor to use.
pub(super) struct TimelineRenderCache {
    pub(super) revision: u64,
    pub(super) path: PathBuf,
    pub(super) from: f64,
    pub(super) to: f64,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) timeline_duration: f64,
}

/// A render still running. Its identity lets Clear Cache silence the worker
/// that was already on its way back to the window.
pub(super) struct TimelineRenderJob {
    pub(super) revision: u64,
    pub(super) path: PathBuf,
}

impl Studio {
    /// The baked clip at `time`, while the cache still describes this exact
    /// document. The render file starts at zero even for an In/Out request,
    /// so the clip's source clock starts at the requested range's beginning.
    pub(super) fn rendered_preview_clip(&self, time: f64) -> Option<Arc<Vec<ExportClip>>> {
        let cache = self.timeline_render.as_ref()?;
        if cache.revision != self.revision
            || !cache.path.is_file()
            || time + 1e-6 < cache.from
            || time > cache.to + 1e-6
        {
            return None;
        }
        let mut clip = ExportClip::blank(
            ClipKind::Video,
            cache.from,
            (cache.to - cache.from).max(0.0),
            0,
        );
        clip.path = cache.path.to_string_lossy().into_owned();
        clip.source_start = cache.from;
        clip.media_width = Some(cache.width);
        clip.media_height = Some(cache.height);
        clip.media_duration = Some(cache.timeline_duration);
        clip.has_audio = Some(false);
        clip.muted = true;
        clip.volume = 0.0;
        Some(Arc::new(vec![clip]))
    }

    /// Render the whole edit once and use either all of it, its entrance up
    /// to the playhead, or its exit from the playhead. Rendering the complete
    /// file keeps transitions, speed curves and animated effects exact; the
    /// range only decides where that file replaces the live compositor.
    pub(super) fn render_timeline(&mut self, action: &str) {
        if self.session.is_none() {
            return;
        }
        let duration = f64::from(self.duration()).max(0.0);
        if self.timeline().clips.is_empty() || duration <= 0.0 {
            self.notify(&t("timelinePane.renderEmpty"), false);
            return;
        }
        let at = f64::from(self.playhead).clamp(0.0, duration);
        let (from, to) = match action {
            "render-in" => (0.0, at),
            "render-out" => (at, duration),
            _ => (0.0, duration),
        };
        if to - from < 1.0 / f64::from(self.frame_rate()).max(1.0) {
            self.notify(&t("timelinePane.renderEmpty"), false);
            return;
        }

        let job = match self.host.exporter.begin() {
            Ok(job) => job,
            Err(_) => {
                self.notify(&t("timelinePane.renderBusy"), false);
                return;
            }
        };
        let revision = self.revision;
        let project = PathBuf::from(
            self.session
                .as_ref()
                .expect("the project was checked")
                .path(),
        );
        let folder = project.join("cache").join("timeline");
        if let Err(error) = std::fs::create_dir_all(&folder) {
            self.notify(
                &format!("{}: {error}", t("timelinePane.renderFailed")),
                true,
            );
            return;
        }
        let path = folder.join(format!("preview-{revision}-{}.mp4", std::process::id()));
        let _ = std::fs::remove_file(&path);

        // A preview cache is intentionally no larger than Half quality. Full
        // resolution belongs to Export; this file is for a fluid monitor.
        let quality = self.quality_of().max(1);
        let (width, height) =
            crate::panes::monitor::MonitorPane::frame_size(quality, self.output_size());
        let hdr = self.timeline().video.color_space.is_hdr();
        let spec = ExportSpec {
            output: path.to_string_lossy().into_owned(),
            crf: 23,
            preset: "veryfast".into(),
            codec: if hdr {
                VideoCodec::Hevc
            } else {
                VideoCodec::H264
            },
            ten_bit: hdr,
            rate_mode: Default::default(),
            bitrate_kbps: 0,
            color_range: ColorRange::Limited,
            hdr,
        };
        let session = self.session.as_ref().expect("the project was checked");
        let titles = self
            .host
            .titles
            .clips(session.project(), width, height)
            .into_iter()
            .map(|title| title.clip)
            .collect();
        let mut request = export::request(session, &spec, titles);
        request.width = width;
        request.height = height;
        // The monitor's live audio engine remains authoritative. Leaving
        // sound out makes the cache quicker and materially smaller.
        for clip in &mut request.clips {
            clip.muted = true;
            clip.volume = 0.0;
            clip.has_audio = Some(false);
        }
        let compositor = self.host.monitor.off_thread();
        self.pause();
        self.timeline_render = None;
        self.timeline_render_job = Some(TimelineRenderJob {
            revision,
            path: path.clone(),
        });
        self.notify(&t("timelinePane.renderStarted"), false);

        let rendered = path.clone();
        spawn_in_project(
            move || {
                let result = export::run_on(&request, compositor, job.cancel_flag(), |_| {});
                (
                    result, rendered, revision, from, to, width, height, duration,
                )
            },
            |studio, _, _, (result, path, revision, from, to, width, height, duration)| {
                let expected = studio
                    .timeline_render_job
                    .as_ref()
                    .is_some_and(|running| running.revision == revision && running.path == path);
                if !expected {
                    let _ = std::fs::remove_file(path);
                    return;
                }
                studio.timeline_render_job = None;
                match result {
                    Ok(_) if studio.revision == revision && path.is_file() => {
                        studio.timeline_render = Some(TimelineRenderCache {
                            revision,
                            path,
                            from,
                            to,
                            width,
                            height,
                            timeline_duration: duration,
                        });
                        studio.notify(&t("timelinePane.renderReady"), false);
                        studio.request_preview();
                    }
                    Ok(_) => {
                        let _ = std::fs::remove_file(path);
                    }
                    Err(error) => {
                        let _ = std::fs::remove_file(path);
                        studio.notify(
                            &format!("{}: {error}", t("timelinePane.renderFailed")),
                            true,
                        );
                    }
                }
            },
        );
    }

    /// Stop a manual render, forget its active file and remove only the
    /// timeline preview cache. Project artwork, waveforms and source proxies
    /// are separate and stay warm.
    pub(super) fn clear_timeline_render_cache(&mut self, notify: bool) {
        if self.timeline_render_job.take().is_some() {
            self.host.exporter.cancel();
        }
        self.timeline_render = None;
        let result = self.session.as_ref().map(|session| {
            let folder = PathBuf::from(session.path()).join("cache").join("timeline");
            if folder.exists() {
                std::fs::remove_dir_all(&folder)
            } else {
                Ok(())
            }
        });
        if notify {
            match result {
                Some(Err(error)) => self.notify(
                    &format!("{}: {error}", t("timelinePane.clearRenderCache")),
                    true,
                ),
                _ => self.notify(&t("timelinePane.renderCacheCleared"), false),
            }
        }
        self.request_preview();
    }
}
