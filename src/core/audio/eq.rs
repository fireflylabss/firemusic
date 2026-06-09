use anyhow::Result;
use libmpv2::Mpv;
use serde::{Deserialize, Serialize};

use crate::core::config::presets_dir;

pub const EQ_BANDS: &[(f64, &str)] = &[
    (31.0, "31"),
    (62.0, "62"),
    (125.0, "125"),
    (250.0, "250"),
    (500.0, "500"),
    (1000.0, "1k"),
    (2000.0, "2k"),
    (4000.0, "4k"),
    (8000.0, "8k"),
    (16000.0, "16k"),
];

const MAX_GAIN: f64 = 12.0;
const MIN_GAIN: f64 = -12.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqPreset {
    pub name: String,
    pub gains: [f64; 10],
}

impl Default for EqPreset {
    fn default() -> Self {
        Self {
            name: "flat".to_string(),
            gains: [0.0; 10],
        }
    }
}

#[derive(Debug, Clone)]
pub struct EqState {
    pub gains: [f64; 10],
    pub preset_name: Option<String>,
    pub selected_band: usize,
}

impl EqState {
    pub fn new() -> Self {
        Self {
            gains: [0.0; 10],
            preset_name: None,
            selected_band: 0,
        }
    }

    pub fn adjust_band(&mut self, delta: f64) {
        self.gains[self.selected_band] =
            (self.gains[self.selected_band] + delta).clamp(MIN_GAIN, MAX_GAIN);
    }

    pub fn reset(&mut self) {
        self.gains = [0.0; 10];
        self.preset_name = None;
    }

    pub fn next_band(&mut self) {
        self.selected_band = (self.selected_band + 1) % 10;
    }

    pub fn prev_band(&mut self) {
        self.selected_band = if self.selected_band == 0 {
            9
        } else {
            self.selected_band - 1
        };
    }

    pub fn apply(&self, mpv: &Mpv) {
        let filters: Vec<String> = self
            .gains
            .iter()
            .enumerate()
            .filter(|(_, g)| g.abs() > 0.01)
            .map(|(i, g)| {
                let freq = EQ_BANDS[i].0;
                format!("equalizer=f={}:width_type=h:width=200:g={}", freq, g)
            })
            .collect();

        if filters.is_empty() {
            mpv.set_property("af", "").ok();
        } else {
            mpv.set_property("af", filters.join(",")).ok();
        }
    }

    pub fn from_preset(preset: &EqPreset) -> Self {
        Self {
            gains: preset.gains,
            preset_name: Some(preset.name.clone()),
            selected_band: 0,
        }
    }

    pub fn to_preset(&self, name: &str) -> EqPreset {
        EqPreset {
            name: name.to_string(),
            gains: self.gains,
        }
    }

    pub fn list_presets() -> Vec<String> {
        let dir = presets_dir();
        if !dir.exists() {
            return Vec::new();
        }
        let mut names = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".json") {
                        names.push(name.trim_end_matches(".json").to_string());
                    }
                }
            }
        }
        names.sort();
        names
    }

    pub fn save_preset(&self, name: &str) -> Result<()> {
        let dir = presets_dir();
        std::fs::create_dir_all(&dir)?;
        let preset = self.to_preset(name);
        let path = dir.join(format!("{}.json", name));
        let json = serde_json::to_string_pretty(&preset)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    pub fn load_preset(name: &str) -> Result<EqState> {
        let path = presets_dir().join(format!("{}.json", name));
        let json = std::fs::read_to_string(&path)?;
        let preset: EqPreset = serde_json::from_str(&json)?;
        Ok(Self::from_preset(&preset))
    }

    pub fn eq_bar(&self, band: usize, width: usize) -> String {
        let gain = self.gains[band];
        let center = width / 2;
        let filled = ((gain / MAX_GAIN) * center as f64).abs() as usize;
        if gain > 0.01 {
            format!(
                "{}{}{}",
                " ".repeat(center),
                "█".repeat(filled),
                " ".repeat(width.saturating_sub(center + filled))
            )
        } else if gain < -0.01 {
            format!(
                "{}{}{}",
                " ".repeat(center.saturating_sub(filled)),
                "▄".repeat(filled),
                " ".repeat(width.saturating_sub(center))
            )
        } else {
            format!("{}{}", " ".repeat(center), " ")
                .repeat(1)
                .chars()
                .take(width)
                .collect()
        }
    }

    pub fn eq_bar_label(&self, band: usize) -> String {
        let gain = self.gains[band];
        let label = EQ_BANDS[band].1;
        if band == self.selected_band {
            format!("{}|{:+5.1}", label, gain)
        } else {
            format!("{} {:+5.1}", label, gain)
        }
    }
}

impl Default for EqState {
    fn default() -> Self {
        Self::new()
    }
}
