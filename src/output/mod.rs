use std::fs::File;
use std::io::Write;
use std::path::Path;

use rusqlite::{params, Connection};
use serde_json::json;

use crate::utils::LogEntry;

pub trait OutputData {
    // TODO! Make return type a Result to check back at caller code
    fn output(
        lines_count: usize,
        error: &[usize],
        sorted_status_code: &[(&u16, &i32)],
        sorted_path: &[(u16, Vec<(&String, &i32)>)],
        entries: &[LogEntry],
        db: Option<Connection>,
    );
}

#[allow(dead_code)]
pub struct JsonOutput;
pub struct DatabaseOutput;

impl OutputData for JsonOutput {
    fn output(
        lines_count: usize,
        error: &[usize],
        sorted_status_code: &[(&u16, &i32)],
        sorted_path: &[(u16, Vec<(&String, &i32)>)],
        // TODO! Remove argument as it will be always None
        _: &[LogEntry],
        _: Option<Connection>,
    ) {
        let mut out = File::create(Path::new("log/log.json")).unwrap();
        let data = json!({
            "total_logs": lines_count,
            "error_logs": error.len(),
            "most_common_error": [
                {
                    "status_code": sorted_status_code[0].0,
                    "frequency": sorted_status_code[0].1,
                    "most_frequent_paths": [
                        {
                            "path": sorted_path.iter().find(|&&(code, _)| code == *sorted_status_code[0].0).unwrap().1[0].0,
                            "frequency": sorted_path.iter().find(|&&(code, _)| code == *sorted_status_code[0].0).unwrap().1[0].1
                        },
                        {
                            "path": sorted_path.iter().find(|&&(code, _)| code == *sorted_status_code[0].0).unwrap().1[1].0,
                            "frequency": sorted_path.iter().find(|&&(code, _)| code == *sorted_status_code[0].0).unwrap().1[1].1
                        },
                        {
                            "path": sorted_path.iter().find(|&&(code, _)| code == *sorted_status_code[0].0).unwrap().1[2].0,
                            "frequency": sorted_path.iter().find(|&&(code, _)| code == *sorted_status_code[0].0).unwrap().1[2].1
                        }
                    ]
                },
                {
                    "status_code": sorted_status_code[1].0,
                    "frequency": sorted_status_code[1].1,
                    "most_frequent_paths": [
                        {
                            "path": sorted_path.iter().find(|&&(code, _)| code == *sorted_status_code[1].0).unwrap().1[0].0,
                            "frequency": sorted_path.iter().find(|&&(code, _)| code == *sorted_status_code[1].0).unwrap().1[0].1
                        },
                        {
                            "path": sorted_path.iter().find(|&&(code, _)| code == *sorted_status_code[1].0).unwrap().1[1].0,
                            "frequency": sorted_path.iter().find(|&&(code, _)| code == *sorted_status_code[1].0).unwrap().1[1].1
                        },
                        {
                            "path": sorted_path.iter().find(|&&(code, _)| code == *sorted_status_code[1].0).unwrap().1[2].0,
                            "frequency": sorted_path.iter().find(|&&(code, _)| code == *sorted_status_code[1].0).unwrap().1[2].1
                        }
                    ]
                },
                {
                    "status_code": sorted_status_code[2].0,
                    "frequency": sorted_status_code[2].1,
                    "most_frequent_paths": [
                        {
                            "path": sorted_path.iter().find(|&&(code, _)| code == *sorted_status_code[2].0).unwrap().1[0].0,
                            "frequency": sorted_path.iter().find(|&&(code, _)| code == *sorted_status_code[2].0).unwrap().1[0].1
                        },
                        {
                            "path": sorted_path.iter().find(|&&(code, _)| code == *sorted_status_code[2].0).unwrap().1[1].0,
                            "frequency": sorted_path.iter().find(|&&(code, _)| code == *sorted_status_code[2].0).unwrap().1[1].1
                        },
                        {
                            "path": sorted_path.iter().find(|&&(code, _)| code == *sorted_status_code[2].0).unwrap().1[2].0,
                            "frequency": sorted_path.iter().find(|&&(code, _)| code == *sorted_status_code[2].0).unwrap().1[2].1
                        }
                    ]
                }
            ],
        });

        // let json = serde_json::to_string(&entries).unwrap();
        // let json = serde_json::to_string(&data).unwrap();
        // out.write_all(json).unwrap();

        out.write_all(data.to_string().as_bytes()).unwrap();
    }
}

// Clenup trait param to only use sorted status ...etc in JsonOutput
impl OutputData for DatabaseOutput {
    fn output(
        _lines_count: usize,
        _error: &[usize],
        _sorted_status_code: &[(&u16, &i32)],
        _sorted_path: &[(u16, Vec<(&String, &i32)>)],
        entries: &[LogEntry],
        db: Option<Connection>,
    ) {
        // !TODO CHECK IF DATABSE ALREADY HAS INFORMATION
        let mut conn = db.unwrap();

        // tx.execute("insert into data (name) values (?1)", ["Sample"])
        // .expect("");
        // tx.execute("insert into data (name) values (?1)", ["S"])
        // .expect("");

        // Optimize for bulk inserts
        conn.execute_batch(
            "
            PRAGMA synchronous = OFF; -- Don't wait for write confirmation
        ",
        )
        .expect("Failed to set PRAGMA");

        // TODO!: LOGS ARE BEING STORE EACH TIME
        let tx = conn.transaction().expect("Failed to create transaction");

        {
            let mut stmt = tx
                .prepare_cached(
                    "INSERT OR REPLACE INTO logs (ip, method, path, status_code, response_size) 
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                )
                .expect("Failed to prepare statement");

            for entry in entries {
                let ip_str = entry.ip.iter().map(|ip| ip.to_string()).collect::<String>();

                stmt.execute(params![
                    ip_str,
                    entry.method,
                    entry.path,
                    entry.status_code,
                    entry.response_size,
                ])
                .expect("Failed to execute statement");
            }
        } // Statement is dropped here

        // Commit the transaction
        tx.commit().expect("Failed to commit transaction");
    }

    // tx.execute(
    //     "INSERT OR REPLACE INTO logs (ip, method, path, status_code, response_size)
    //          VALUES (?1, ?2, ?3, ?4, ?5)",
    //     params![
    //         ipStr, // Convert IpAddr to string
    //         // entry.timestamp,
    //         entry.method,
    //         entry.path,
    //         entry.status_code,
    //         entry.response_size,
    //     ],
    // )
    // .expect("Error in the query to db");

    // tx.execute(
    //     "INSERT INTO logs (ip, timestamp, method, path, status_code, response_size)
    //      VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    //     params![
    //         ipStr, // Convert IpAddr to string
    //         // entry.timestamp,
    //         entry.method,
    //         entry.path,
    //         entry.status_code,
    //         entry.response_size,
    //     ],
    // )
    // .expect("Error in the query to db");
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn get_dummy_entries() -> Vec<LogEntry> {
        vec![
            LogEntry {
                ip: [192, 168, 1, 1, 0, 0],
                timestamp: Utc::now(),
                method: "GET".to_string(),
                path: "/index.html".to_string(),
                protocol: "HTTP/1.1".to_string(),
                status_code: 200,
                response_size: 1024,
            },
            LogEntry {
                ip: [10, 0, 0, 1, 0, 0],
                timestamp: Utc::now(),
                method: "POST".to_string(),
                path: "/submit".to_string(),
                protocol: "HTTP/1.1".to_string(),
                status_code: 404,
                response_size: 512,
            },
        ]
    }

    #[test]
    fn test_database_output() {
        // Use a named temp file to pass a valid connection that persists out of this scope
        // to check side effects, or use the in memory and we just check it runs without panicking.
        let temp_db = tempfile::NamedTempFile::new().unwrap();
        let conn = Connection::open(temp_db.path()).unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS logs (
                id INTEGER PRIMARY KEY,
                ip TEXT NOT NULL,
                method TEXT NOT NULL,
                path TEXT NOT NULL,
                status_code INTEGER NOT NULL,
                response_size INTEGER NOT NULL
            )",
            [],
        )
        .unwrap();

        let entries = get_dummy_entries();
        let error_codes = vec![];
        let sorted_status_code = vec![];
        let sorted_path = vec![];

        DatabaseOutput::output(
            2,
            &error_codes,
            &sorted_status_code,
            &sorted_path,
            &entries,
            Some(conn), // ownership transferred, but in test it's fine
        );

        // Let's verify it worked by opening a new connection to the same file
        let conn2 = Connection::open(temp_db.path()).unwrap();
        let count: i32 = conn2.query_row("SELECT count(*) FROM logs", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_json_output() {
        // Prepare dummy data structure
        let p1 = String::from("/test1");
        let p2 = String::from("/test2");
        let p3 = String::from("/test3");

        let path1_data = vec![(&p1, &5), (&p2, &3), (&p3, &1)];
        let path2_data = vec![(&p1, &4), (&p2, &2), (&p3, &1)];
        let path3_data = vec![(&p1, &3), (&p2, &2), (&p3, &1)];

        let error_codes = vec![1, 2];
        let code_404: u16 = 404;
        let code_500: u16 = 500;
        let code_403: u16 = 403;

        let freq_404: i32 = 10;
        let freq_500: i32 = 5;
        let freq_403: i32 = 2;

        let sorted_status_code = vec![
            (&code_404, &freq_404),
            (&code_500, &freq_500),
            (&code_403, &freq_403)
        ];

        let sorted_path = vec![
            (code_404, path1_data),
            (code_500, path2_data),
            (code_403, path3_data),
        ];

        let entries = get_dummy_entries();

        std::fs::create_dir_all("log").unwrap();

        JsonOutput::output(
            100,
            &error_codes,
            &sorted_status_code,
            &sorted_path,
            &entries,
            None,
        );

        let path = Path::new("log/log.json");
        assert!(path.exists());

        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("\"total_logs\":100"));
        assert!(content.contains("\"error_logs\":2"));
        assert!(content.contains("\"status_code\":404"));

        std::fs::remove_file(path).unwrap();
    }
}
