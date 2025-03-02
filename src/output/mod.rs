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
        error: &Vec<usize>,
        sorted_status_code: &Vec<(&u16, &i32)>,
        sorted_path: &Vec<(u16, Vec<(&String, &i32)>)>,
        entries: &Vec<LogEntry>,
        db: Option<Connection>,
    );
}

pub struct JsonOutput;
pub struct DatabaseOutput;

impl OutputData for JsonOutput {
    fn output(
        lines_count: usize,
        error: &Vec<usize>,
        sorted_status_code: &Vec<(&u16, &i32)>,
        sorted_path: &Vec<(u16, Vec<(&String, &i32)>)>,
        // TODO! Remove argument as it will be always None
        _: &Vec<LogEntry>,
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
        _error: &Vec<usize>,
        _sorted_status_code: &Vec<(&u16, &i32)>,
        _sorted_path: &Vec<(u16, Vec<(&String, &i32)>)>,
        entries: &Vec<LogEntry>,
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
