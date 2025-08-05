use std::fs::File;
use std::io::Write;
use std::path::Path;

use rusqlite::{params, Connection};
use serde_json::json;

use crate::utils::LogEntry;

/// Trait for outputting processed log analysis data
pub trait OutputData {
    /// Output analysis results in a specific format
    /// 
    /// # Arguments
    /// * `lines_count` - Total number of log lines processed
    /// * `error_indices` - Indices of log entries that had errors
    /// * `sorted_status_codes` - Status codes sorted by frequency
    /// * `sorted_paths` - Paths grouped by status code, sorted by frequency
    /// * `entries` - All parsed log entries
    /// * `db` - Optional database connection for database output
    fn output(
        lines_count: usize,
        error_indices: &[usize],
        sorted_status_codes: &[(&u16, &i32)],
        sorted_paths: &[(u16, Vec<(&String, &i32)>)],
        entries: &[LogEntry],
        db: Option<Connection>,
    );
}

/// JSON output implementation
pub struct JsonOutput;

/// Database output implementation  
pub struct DatabaseOutput;

impl OutputData for JsonOutput {
    fn output(
        lines_count: usize,
        error_indices: &[usize],
        sorted_status_codes: &[(&u16, &i32)],
        sorted_paths: &[(u16, Vec<(&String, &i32)>)],
        _entries: &[LogEntry], // Unused for JSON output
        _db: Option<Connection>, // Unused for JSON output
    ) {
        let mut output_file = File::create(Path::new("log/log.json")).unwrap();
        
        let json_data = json!({
            "total_logs": lines_count,
            "error_logs": error_indices.len(),
            "most_common_errors": sorted_status_codes.iter().take(3).map(|(code, count)| {
                let most_frequent_paths = if let Some((_, paths)) = sorted_paths
                    .iter()
                    .find(|(status_code, _)| *status_code == **code)
                {
                    paths.iter().take(3).map(|(path, freq)| {
                        json!({
                            "path": path,
                            "frequency": freq
                        })
                    }).collect::<Vec<_>>()
                } else {
                    Vec::new()
                };
                    
                json!({
                    "status_code": code,
                    "frequency": count,
                    "most_frequent_paths": most_frequent_paths
                })
            }).collect::<Vec<_>>()
        });

        output_file.write_all(json_data.to_string().as_bytes()).unwrap();
    }
}

impl OutputData for DatabaseOutput {
    fn output(
        _lines_count: usize,
        _error_indices: &[usize],
        _sorted_status_codes: &[(&u16, &i32)],
        _sorted_paths: &[(u16, Vec<(&String, &i32)>)],
        entries: &[LogEntry],
        db: Option<Connection>,
    ) {
        let mut conn = db.expect("Database connection required for DatabaseOutput");

        // Optimize for bulk inserts
        conn.execute_batch("PRAGMA synchronous = OFF;")
            .expect("Failed to set PRAGMA");

        let tx = conn.transaction().expect("Failed to create transaction");

        {
            let mut stmt = tx
                .prepare_cached(
                    "INSERT OR REPLACE INTO logs (ip, method, path, status_code, response_size) 
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                )
                .expect("Failed to prepare statement");

            for entry in entries {
                let ip_string = entry.ip
                    .iter()
                    .map(|segment| segment.to_string())
                    .collect::<Vec<String>>()
                    .join(".");

                stmt.execute(params![
                    ip_string,
                    entry.method,
                    entry.path,
                    entry.status_code,
                    entry.response_size,
                ])
                .expect("Failed to execute statement");
            }
        }

        tx.commit().expect("Failed to commit transaction");
    }
}
