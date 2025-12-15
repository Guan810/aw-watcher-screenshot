use anyhow::{Result, anyhow};
use std::sync::{Arc, LazyLock, RwLock};
use xcap::Monitor;

use crate::config::MonitorConfig;

#[derive(Clone)]
pub struct MonitorCapture {
    monitor: HashMap<String, SafeMonitor>,
}

#[derive(Clone)]
pub struct SafeMonitor {
    id: String,
    config: MonitorConfig,
    monitor: Arc<RwLock<Monitor>>,
}

static RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?P<name>.+)_(?P<w>\d+)_(?P<h>\d+)_(?P<x>-?\d+)_(?P<y>-?\d+)").unwrap()
});

impl MonitorCapture {
    pub fn new(monitorConfigs: HashMap<String, MonitorConfig>) -> Self {
        let mut monitors = HashMap::new();
        for (monitor_id, config) in monitorConfigs {
            let monitor = SafeMonitor::new(monitor_id).unwrap();
            monitors.insert(monitor_id, monitor);
        }
        Self { monitors }
    }

    fn capture_loop(monitor_id: String) -> Result<Vec<Vec<u8>>> {
        let monitor = SafeMonitor::new(monitor_id, Default::default()).unwrap();
        let mut captures = Vec::new();
        loop {
            let capture = monitor.capture()?;
            captures.push(capture);
            std::thread::sleep(std::time::Duration::from_millis(monitor.config.interval));
        }
    }
}

impl SafeMonitor {
    pub fn new(monitor_id: String, config: MonitorConfig) -> Result<Self> {
        let (name, width, height, x, y) = Self::parse_id(&monitor_id)?;
        let monitor = Monitor::from_point(x, y)?;

        if monitor.name() != name && monitor.width() != width && monitor.height() != height {
            return Err(anyhow!("Monitor name mismatch"));
        }

        Ok(SafeMonitor {
            monitor_id,
            config,
            monitor: Arc::new(RwLock::new(monitor)),
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

    pub fn with_monitor<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&Monitor) -> Result<R>,
    {
        let monitor = self.monitor.read().unwrap();
        f(&monitor)
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
