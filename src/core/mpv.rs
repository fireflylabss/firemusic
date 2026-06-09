use libmpv2::Mpv;

use super::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct MpvConfig {
    pub volume: f64,
    pub speed: f64,
    pub loop_mode: bool,
    pub crossfade: f64,
    pub ytdl_format: Option<String>,
}

impl MpvConfig {
    pub fn for_cli(volume: f64, speed: f64, loop_mode: bool, crossfade: f64) -> Self {
        Self {
            volume: volume.clamp(0.0, 100.0),
            speed: speed.clamp(0.1, 10.0),
            loop_mode,
            crossfade: crossfade.max(0.0),
            ytdl_format: None,
        }
    }

    pub fn for_stream(volume: f64, speed: f64, loop_mode: bool, ytdl_format: &str) -> Self {
        Self {
            volume: volume.clamp(0.0, 100.0),
            speed: speed.clamp(0.1, 10.0),
            loop_mode,
            crossfade: 0.0,
            ytdl_format: Some(ytdl_format.to_string()),
        }
    }
}

pub fn create_player(config: &MpvConfig) -> Result<Mpv> {
    let mpv = Mpv::new().map_err(|e| Error::MpvInit(format!("{:?}", e)))?;

    mpv.set_property("video", "no").ok();
    mpv.set_property("volume", config.volume).ok();
    mpv.set_property("speed", config.speed).ok();
    mpv.set_property("ytdl", "yes").ok();

    let ytdl_format = config
        .ytdl_format
        .as_deref()
        .unwrap_or("bestaudio/best");
    mpv.set_property("ytdl-format", ytdl_format).ok();

    if config.loop_mode {
        mpv.set_property("loop-file", "inf").ok();
    }

    if config.crossfade > 0.0 {
        mpv.set_property("audio-fade", config.crossfade).ok();
    }

    Ok(mpv)
}

pub fn load_inputs(mpv: &Mpv, inputs: &[String]) -> Result<()> {
    for (i, input) in inputs.iter().enumerate() {
        let mode = if i == 0 { "replace" } else { "append" };
        mpv.command("loadfile", &[input.as_str(), mode])
            .map_err(|e| Error::MpvCommand(format!("failed to load {}: {:?}", input, e)))?;
    }
    Ok(())
}