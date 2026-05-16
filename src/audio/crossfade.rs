#[derive(Debug, Clone)]
pub struct CrossfadeConfig {
    #[allow(dead_code)]
    pub enabled: bool,
    pub duration: f64,
}

impl CrossfadeConfig {
    pub fn new(duration: f64) -> Self {
        Self {
            enabled: duration > 0.0,
            duration: duration.max(0.0),
        }
    }

    pub fn disabled() -> Self {
        Self {
            enabled: false,
            duration: 0.0,
        }
    }
}
