use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use std::sync::LazyLock;
use xcap::Monitor;

use crate::capture::utils::hamming_distance;
use crate::config::MonitorConfig;
use crate::event::CaptureResult;

pub struct SafeMonitor {
    id: String,
    config: MonitorConfig,
    monitor: Monitor,

    last_capture_time: Option<DateTime<Utc>>,
    last_capture_dhash: Option<u64>,
}

static RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?P<name>.+)_(?P<w>\d+)_(?P<h>\d+)_(?P<x>-?\d+)_(?P<y>-?\d+)")
        .expect("invalid regex")
});

impl SafeMonitor {
    pub fn new(monitor_id: String, config: MonitorConfig) -> Result<Self> {
        let (name, width, height, x, y) = Self::parse_id(&monitor_id)?;
        let monitor = Monitor::from_point(x, y)?;

        let monitor_name = monitor.name()?;
        let monitor_width = monitor.width()?;
        let monitor_height = monitor.height()?;

        if monitor_name != name || monitor_width != width || monitor_height != height {
            return Err(anyhow!(
                "Monitor mismatch: expected {}x{} at ({},{}), got {}x{}",
                width,
                height,
                x,
                y,
                monitor_width,
                monitor_height
            ));
        }

        Ok(SafeMonitor {
            id: monitor_id,
            config,
            monitor: monitor,
            last_capture_time: None,
            last_capture_dhash: None,
        })
    }

    fn parse_id(monitor_id: &str) -> Result<(String, u32, u32, i32, i32)> {
        let cap = RE
            .captures(monitor_id)
            .ok_or_else(|| anyhow!("Invalid monitor ID format: {}", monitor_id))?;
        let name = cap["name"].to_string();
        let width = cap["w"].parse()?;
        let height = cap["h"].parse()?;
        let x = cap["x"].parse()?;
        let y = cap["y"].parse()?;
        Ok((name, width, height, x, y))
    }

    pub fn capture_once(&mut self) -> Result<Option<CaptureResult>> {
        let now = Utc::now();

        let image = self
            .monitor
            .capture_image()
            .map_err(|_| anyhow!("Failed to capture image"))?;

        let dhash = crate::capture::utils::dHash(&image, self.config.dhash_resolution);

        if let Some(last_time) = self.last_capture_time {
            if let Some(ref last_hash) = self.last_capture_dhash {
                let delta = (now - last_time).num_milliseconds() as u64;
                let time_too_soon = delta < self.config.enforce_interval;

                let hash_too_similar =
                    hamming_distance(dhash, last_hash.clone()) < self.config.dhash_threshold;
                if time_too_soon && hash_too_similar {
                    return Ok(None);
                }
            }
        }

        self.last_capture_time = Some(now);
        self.last_capture_dhash = Some(dhash);

        Ok(Some(CaptureResult::new(self.id.clone(), image, now)))
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_id() {
        let (name, width, height, x, y) = SafeMonitor::parse_id("monitor_1920_1080_0_0").unwrap();
        assert_eq!(name, "monitor");
        assert_eq!(width, 1920);
        assert_eq!(height, 1080);
        assert_eq!(x, 0);
        assert_eq!(y, 0);
    }
}
