use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::Sender;

use crate::capture::monitor::SafeMonitor;
use crate::config::MonitorConfig;
use crate::event::CaptureResult;

pub struct MonitorCapture {
    monitors: HashMap<String, Arc<RwLock<SafeMonitor>>>,
}

impl MonitorCapture {
    pub fn new(monitor_configs: HashMap<String, MonitorConfig>) -> Self {
        let mut monitors = HashMap::new();
        for (monitor_id, config) in monitor_configs {
            if config.enable {
                let monitor = SafeMonitor::new(monitor_id.clone(), config)
                    .expect("Failed init MonitorCapture.");
                monitors.insert(monitor_id, Arc::new(RwLock::new(monitor)));
            }
        }
        Self { monitors }
    }

    pub fn start_capture(&self, sender: Sender<CaptureResult>) {
        for (monitor_id, monitor) in &self.monitors {
            let monitor = Arc::clone(monitor);
            let monitor_id = monitor_id.clone();
            let sender = sender.clone();

            tokio::spawn(async move {
                loop {
                    let result = {
                        let mut m = match monitor.write() {
                            Ok(m) => m,
                            Err(e) => {
                                eprintln!("Failed to write monitor {}: {}", monitor_id, e);
                                continue;
                            }
                        };

                        match m.capture_once() {
                            Ok(Some(event)) => Some(CaptureResult {
                                monitor_id: monitor_id.clone(),
                                image: image::DynamicImage::ImageRgba8(image),
                                timestamp,
                            }),
                            Ok(None) => None,
                            Err(e) => {
                                eprintln!("Failed to capture image: {}", e);
                                None
                            }
                        }
                    };

                    if let Some(result) = result {
                        let _ = sender.send(result).await;
                    }
                }
            });
        }
    }
}
