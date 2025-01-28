use std::fs::File;
use std::io::Write;

use serde_json::{json, Result};

pub trait OutputData {
    fn output(
        lines_count: usize,
        error: &Vec<usize>,
        sorted_status_code: &Vec<(&u16, &i32)>,
        sorted_path: &Vec<(u16, Vec<(&String, &i32)>)>,
    );
}

pub struct JsonOutput;

impl OutputData for JsonOutput {
    fn output(
        lines_count: usize,
        error: &Vec<usize>,
        sorted_status_code: &Vec<(&u16, &i32)>,
        sorted_path: &Vec<(u16, Vec<(&String, &i32)>)>,
    ) {
        let mut out = File::create("log/log.json").unwrap();
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
