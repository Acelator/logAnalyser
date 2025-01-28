use std::fs::File;
use std::io::Write;

use serde_json::{json, Result};

pub trait OutputData {
    fn output(
        linesCount: usize,
        error: &Vec<usize>,
        sortedStatusCode: &Vec<(&u16, &i32)>,
        sortedPath: &Vec<(u16, Vec<(&String, &i32)>)>,
    );
}

pub struct JsonOutput;

impl OutputData for JsonOutput {
    fn output(
        linesCount: usize,
        error: &Vec<usize>,
        sortedStatusCode: &Vec<(&u16, &i32)>,
        sortedPath: &Vec<(u16, Vec<(&String, &i32)>)>,
    ) {
        let mut out = File::create("log/log.json").unwrap();
        let data = json!({
            "total_logs": linesCount,
            "error_logs": error.len(),
            "most_common_error": [
                {
                    "status_code": sortedStatusCode[0].0,
                    "frequency": sortedStatusCode[0].1,
                    "most_frequent_paths": [
                        {
                            "path": sortedPath.iter().find(|&&(code, _)| code == *sortedStatusCode[0].0).unwrap().1[0].0,
                            "frequency": sortedPath.iter().find(|&&(code, _)| code == *sortedStatusCode[0].0).unwrap().1[0].1
                        },
                        {
                            "path": sortedPath.iter().find(|&&(code, _)| code == *sortedStatusCode[0].0).unwrap().1[1].0,
                            "frequency": sortedPath.iter().find(|&&(code, _)| code == *sortedStatusCode[0].0).unwrap().1[1].1
                        },
                        {
                            "path": sortedPath.iter().find(|&&(code, _)| code == *sortedStatusCode[0].0).unwrap().1[2].0,
                            "frequency": sortedPath.iter().find(|&&(code, _)| code == *sortedStatusCode[0].0).unwrap().1[2].1
                        }
                    ]
                },
                {
                    "status_code": sortedStatusCode[1].0,
                    "frequency": sortedStatusCode[1].1,
                    "most_frequent_paths": [
                        {
                            "path": sortedPath.iter().find(|&&(code, _)| code == *sortedStatusCode[1].0).unwrap().1[0].0,
                            "frequency": sortedPath.iter().find(|&&(code, _)| code == *sortedStatusCode[1].0).unwrap().1[0].1
                        },
                        {
                            "path": sortedPath.iter().find(|&&(code, _)| code == *sortedStatusCode[1].0).unwrap().1[1].0,
                            "frequency": sortedPath.iter().find(|&&(code, _)| code == *sortedStatusCode[1].0).unwrap().1[1].1
                        },
                        {
                            "path": sortedPath.iter().find(|&&(code, _)| code == *sortedStatusCode[1].0).unwrap().1[2].0,
                            "frequency": sortedPath.iter().find(|&&(code, _)| code == *sortedStatusCode[1].0).unwrap().1[2].1
                        }
                    ]
                },
                {
                    "status_code": sortedStatusCode[2].0,
                    "frequency": sortedStatusCode[2].1,
                    "most_frequent_paths": [
                        {
                            "path": sortedPath.iter().find(|&&(code, _)| code == *sortedStatusCode[2].0).unwrap().1[0].0,
                            "frequency": sortedPath.iter().find(|&&(code, _)| code == *sortedStatusCode[2].0).unwrap().1[0].1
                        },
                        {
                            "path": sortedPath.iter().find(|&&(code, _)| code == *sortedStatusCode[2].0).unwrap().1[1].0,
                            "frequency": sortedPath.iter().find(|&&(code, _)| code == *sortedStatusCode[2].0).unwrap().1[1].1
                        },
                        {
                            "path": sortedPath.iter().find(|&&(code, _)| code == *sortedStatusCode[2].0).unwrap().1[2].0,
                            "frequency": sortedPath.iter().find(|&&(code, _)| code == *sortedStatusCode[2].0).unwrap().1[2].1
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
