use anyhow::Result;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

pub struct QASaveTrigger {
    current_font: Option<PathBuf>,
    last_analysis_hash: Option<String>,
}

impl Default for QASaveTrigger {
    fn default() -> Self {
        Self::new()
    }
}

impl QASaveTrigger {
    pub fn new() -> Self {
        Self {
            current_font: None,
            last_analysis_hash: None,
        }
    }

    pub fn set_current_font(&mut self, font_path: PathBuf) {
        self.current_font = Some(font_path);
        // Reset analysis hash when font changes
        self.last_analysis_hash = None;
    }

    pub fn should_run_qa(&mut self, trigger_event: QATriggerEvent) -> Result<bool> {
        match trigger_event {
            QATriggerEvent::FontSaved => {
                // Always run QA when font is saved
                Ok(true)
            }
            QATriggerEvent::FontExported => {
                // Always run QA when font is exported
                Ok(true)
            }
            QATriggerEvent::ManualRefresh => {
                // Always run QA when manually requested
                Ok(true)
            }
            QATriggerEvent::FontChanged => {
                // Check if font content has actually changed
                if let Some(ref font_path) = self.current_font {
                    let current_hash = self.calculate_content_hash(font_path)?;
                    let should_run = Some(&current_hash) != self.last_analysis_hash.as_ref();
                    if should_run {
                        self.last_analysis_hash = Some(current_hash);
                    }
                    Ok(should_run)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn calculate_content_hash(&self, font_path: &PathBuf) -> Result<String> {
        let mut hasher = DefaultHasher::new();

        // Hash the modification time
        let metadata = std::fs::metadata(font_path)?;
        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                duration.as_secs().hash(&mut hasher);
            }
        }

        // Hash file size
        metadata.len().hash(&mut hasher);

        Ok(format!("{:x}", hasher.finish()))
    }

    pub fn get_current_font(&self) -> Option<&PathBuf> {
        self.current_font.as_ref()
    }

    pub fn clear_current_font(&mut self) {
        self.current_font = None;
        self.last_analysis_hash = None;
    }
}

#[derive(Debug, Clone)]
pub enum QATriggerEvent {
    FontSaved,
    FontExported,
    ManualRefresh,
    FontChanged,
}

impl QATriggerEvent {
    pub fn description(&self) -> &'static str {
        match self {
            QATriggerEvent::FontSaved => "Font saved",
            QATriggerEvent::FontExported => "Font exported",
            QATriggerEvent::ManualRefresh => "Manual refresh",
            QATriggerEvent::FontChanged => "Font content changed",
        }
    }
}
