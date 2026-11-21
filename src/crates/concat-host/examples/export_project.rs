use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use concat_host::export::{self, ExportSpec};
use concat_host::projects;
use concat_host::session::Session;
use concat_media::{ColorRange, RateMode, VideoCodec};

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            eprintln!("{} {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: Logger = Logger;

fn main() -> Result<(), String> {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.len() < 4 {
        return Err("usage: export_project PROJECT OUTPUT SECONDS [auto|software|qsv|mf]".into());
    }
    let project = &arguments[1];
    let output = &arguments[2];
    let seconds: f64 = arguments[3]
        .parse()
        .map_err(|_| "SECONDS must be a number".to_owned())?;
    let mode = arguments.get(4).map(String::as_str).unwrap_or("auto");

    log::set_logger(&LOGGER).map_err(|error| error.to_string())?;
    log::set_max_level(log::LevelFilter::Info);
    match mode {
        "auto" => {}
        "software" => unsafe { std::env::set_var("CONCAT_HARDWARE_ENCODE", "0") },
        "qsv" => unsafe { std::env::set_var("CONCAT_ENCODER", "h264_qsv") },
        "mf" => unsafe { std::env::set_var("CONCAT_ENCODER", "h264_mf") },
        _ => return Err(format!("unknown encoder mode: {mode}")),
    }

    let info = projects::open(project)?;
    let session = Session::open_info(&info)?;
    let spec = ExportSpec {
        output: output.clone(),
        crf: 20,
        preset: "veryfast".into(),
        codec: VideoCodec::H264,
        ten_bit: false,
        rate_mode: RateMode::Average,
        bitrate_kbps: 12_000,
        color_range: ColorRange::Limited,
        hdr: false,
    };
    let mut request = export::request(&session, &spec, Vec::new());
    request.clips.retain(|clip| clip.start < seconds);
    for clip in &mut request.clips {
        clip.duration = clip.duration.min((seconds - clip.start).max(0.0));
    }
    let cancel = AtomicBool::new(false);
    let started = Instant::now();
    let result = export::run(&request, &cancel, |progress| {
        eprintln!(
            "PROGRESS stage={} frame={} total={}",
            progress.stage, progress.frame, progress.total
        );
    });
    eprintln!(
        "BENCHMARK mode={mode} seconds={seconds:.3} elapsed_s={:.3} output={} exists={}",
        started.elapsed().as_secs_f64(),
        output,
        Path::new(output).is_file()
    );
    result.map(|_| ())
}
