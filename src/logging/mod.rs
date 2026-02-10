use crate::app::state::{BufferKey, Message, MessageKind};
use crate::config::LoggingConfig;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub struct ChatLogger {
    enabled: bool,
    log_dir: String,
    log_channels: bool,
    log_queries: bool,
    file_handles: HashMap<String, fs::File>,
}

impl ChatLogger {
    pub fn new(config: &LoggingConfig) -> Self {
        Self {
            enabled: config.enabled,
            log_dir: config.log_dir.clone(),
            log_channels: config.log_channels,
            log_queries: config.log_queries,
            file_handles: HashMap::new(),
        }
    }

    pub fn log_message(&mut self, key: &BufferKey, msg: &Message) {
        if !self.enabled {
            return;
        }

        let target = match key {
            BufferKey::Channel(_, ch) if self.log_channels => ch.clone(),
            BufferKey::Query(_, nick) if self.log_queries => nick.clone(),
            _ => return,
        };

        let line = match msg.kind {
            MessageKind::Normal | MessageKind::Notice => {
                format!("[{}] <{}> {}", msg.timestamp, msg.sender, msg.text)
            }
            MessageKind::Action => {
                format!("[{}] * {} {}", msg.timestamp, msg.sender, msg.text)
            }
            MessageKind::Join | MessageKind::Part | MessageKind::Quit | MessageKind::System => {
                format!("[{}] *** {} {}", msg.timestamp, msg.sender, msg.text)
            }
            MessageKind::Error => {
                format!("[{}] !!! {}", msg.timestamp, msg.text)
            }
        };

        // Sanitize target for filename
        let safe_target: String = target
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '_' })
            .collect();

        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let filename = format!("{}_{}.log", safe_target, date);

        // Expand ~ in log_dir
        let log_dir = if self.log_dir.starts_with('~') {
            if let Some(home) = dirs::home_dir() {
                home.join(&self.log_dir[2..])
            } else {
                PathBuf::from(&self.log_dir)
            }
        } else {
            PathBuf::from(&self.log_dir)
        };

        let filepath = log_dir.join(&filename);

        // Get or create file handle
        let handle = self.file_handles.entry(filename.clone()).or_insert_with(|| {
            let _ = fs::create_dir_all(&log_dir);
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&filepath)
                .unwrap_or_else(|_| {
                    // Fallback: create a temp file that goes nowhere
                    OpenOptions::new()
                        .write(true)
                        .open(if cfg!(unix) { "/dev/null" } else { "NUL" })
                        .unwrap()
                })
        });

        let _ = writeln!(handle, "{}", line);
    }
}
