use crate::qa::{Category, QAIssue, QAReport, QASummary, Severity};
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::Path;
use std::time::SystemTime;
use tempfile::NamedTempFile;
use tokio::process::Command;

pub struct FontspectorRunner {
    profile: FontspectorProfile,
}

#[derive(Debug, Clone)]
#[derive(Default)]
pub enum FontspectorProfile {
    #[default]
    Universal,
    OpenType,
}

impl FontspectorRunner {
    pub fn new() -> Result<Self> {
        Ok(Self {
            profile: FontspectorProfile::Universal,
        })
    }

    pub fn with_profile(profile: FontspectorProfile) -> Self {
        Self { profile }
    }

    pub async fn analyze(&self, font_path: &Path) -> Result<QAReport> {
        // Create temporary file for JSON output
        let temp_file = NamedTempFile::new()
            .context("Failed to create temporary file for Fontspector output")?;
        let json_path = temp_file.path();

        // Determine profile string for Fontspector CLI
        let profile_str = match self.profile {
            FontspectorProfile::Universal => "universal",
            FontspectorProfile::OpenType => "opentype",
        };

        // Run Fontspector with JSON output
        let output = Command::new("fontspector")
            .arg("--json")
            .arg(json_path)
            .arg("--profile")
            .arg(profile_str)
            .arg(font_path)
            .output()
            .await
            .context("Failed to execute fontspector command")?;

        // Fontspector returns non-zero exit status when there are FAIL-level issues
        // This is normal behavior, so we only error if there's an actual execution problem
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Only treat as error if stderr contains actual error messages or if status is not 1
            if !stderr.trim().is_empty() || output.status.code() != Some(1) {
                return Err(anyhow::anyhow!(
                    "Fontspector failed with status {}: {}",
                    output.status,
                    stderr
                ));
            }
            // Status 1 with empty stderr is normal when there are FAIL-level issues
        }

        // Read and parse JSON output
        let json_content = tokio::fs::read_to_string(json_path)
            .await
            .context("Failed to read Fontspector JSON output")?;

        let json: Value = serde_json::from_str(&json_content)
            .context("Failed to parse Fontspector JSON output")?;

        // Parse summary
        let summary = self.parse_summary(&json)?;

        // Parse issues
        let issues = self.parse_issues(&json)?;

        Ok(QAReport {
            font_path: font_path.to_path_buf(),
            timestamp: SystemTime::now(),
            issues,
            summary,
        })
    }

    fn parse_summary(&self, json: &Value) -> Result<QASummary> {
        let summary_obj = json["summary"]
            .as_object()
            .context("Missing or invalid summary in Fontspector output")?;

        let skip = summary_obj
            .get("SKIP")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let pass = summary_obj
            .get("PASS")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let info = summary_obj
            .get("INFO")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let warn = summary_obj
            .get("WARN")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let fail = summary_obj
            .get("FAIL")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let error = summary_obj
            .get("ERROR")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        Ok(QASummary {
            total_checks: (skip + pass + info + warn + fail + error) as usize,
            passed: pass as usize,
            failed: (fail + error) as usize,
            warnings: warn as usize,
            info: info as usize,
            skipped: skip as usize,
        })
    }

    fn parse_issues(&self, json: &Value) -> Result<Vec<QAIssue>> {
        let mut issues = Vec::new();

        if let Some(results) = json["results"].as_object() {
            for (_, file_results) in results {
                if let Some(file_obj) = file_results.as_object() {
                    for (_, section_checks) in file_obj {
                        if let Some(checks_array) = section_checks.as_array() {
                            for check in checks_array {
                                self.parse_check_issues(check, &mut issues)?;
                            }
                        }
                    }
                }
            }
        }

        Ok(issues)
    }

    fn parse_check_issues(&self, check: &Value, issues: &mut Vec<QAIssue>) -> Result<()> {
        let check_id = check["check_id"].as_str().unwrap_or("unknown").to_string();

        if let Some(subresults) = check["subresults"].as_array() {
            for subresult in subresults {
                let severity_str = subresult["severity"].as_str().unwrap_or("UNKNOWN");

                // Only include issues that are not PASS or SKIP
                if matches!(severity_str, "FAIL" | "ERROR" | "WARN" | "INFO") {
                    let severity = match severity_str {
                        "FAIL" | "ERROR" => Severity::Error,
                        "WARN" => Severity::Warning,
                        "INFO" => Severity::Info,
                        _ => Severity::Info,
                    };

                    let message = subresult["message"]
                        .as_str()
                        .unwrap_or("No message provided")
                        .to_string();

                    let category = Self::categorize_check(&check_id);

                    // Extract location information if available
                    let location = self.extract_location(&message, &check_id);

                    issues.push(QAIssue {
                        severity,
                        category,
                        check_id: check_id.clone(),
                        message,
                        location,
                    });
                }
            }
        }

        Ok(())
    }

    fn extract_location(&self, message: &str, check_id: &str) -> Option<crate::qa::Location> {
        // Try to extract glyph names from common message patterns
        let glyph_name = if message.contains("glyph") {
            // Look for patterns like "glyph 'name'" or "glyph ('name')"
            if let Some(start) = message.find("'") {
                message[start + 1..].find("'").map(|end| message[start + 1..start + 1 + end].to_string())
            } else {
                None
            }
        } else {
            None
        };

        // Extract table name from check ID or message
        let table_name = if check_id.contains("os2") || message.contains("OS/2") {
            Some("OS/2".to_string())
        } else if check_id.contains("hhea") || message.contains("hhea") {
            Some("hhea".to_string())
        } else if check_id.contains("name") || message.contains("name table") {
            Some("name".to_string())
        } else if check_id.contains("gdef") || message.contains("GDEF") {
            Some("GDEF".to_string())
        } else {
            None
        };

        if glyph_name.is_some() || table_name.is_some() {
            Some(crate::qa::Location {
                glyph_name,
                table_name,
                position: None, // Fontspector doesn't provide coordinate positions
            })
        } else {
            None
        }
    }

    fn categorize_check(check_id: &str) -> Category {
        if check_id.contains("outline")
            || check_id.contains("contour")
            || check_id.contains("glyph")
        {
            Category::Outlines
        } else if check_id.contains("meta")
            || check_id.contains("name")
            || check_id.contains("info")
        {
            Category::Metadata
        } else if check_id.contains("hint") {
            Category::Hinting
        } else if check_id.contains("kern") {
            Category::Kerning
        } else if check_id.contains("spacing") || check_id.contains("width") {
            Category::Spacing
        } else if check_id.contains("unicode") || check_id.contains("codepoint") {
            Category::Unicode
        } else {
            Category::Other(check_id.to_string())
        }
    }
}

