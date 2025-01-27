// TODO: ADD a formatter before processing the log file
// TODO: VALIDATE LOG
// TODO: ADD ERRORS

use parser::{ApacheLogPaser, LogParser};
use sysinfo::System;

use serde_json::{json, Result};

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};

mod parser;
mod utils;

fn main() {
    let mut sys = System::new_all();

    let logLevel = ["INFO", "WARNING", "ERROR", "CRITICAL"];
    let parser = ApacheLogPaser;

    let mut entries: Vec<utils::LogEntry> = Vec::new();

    //let mut log = LogEntry {ip: [255,255,255,255,255,255], timestamp: Utc::now(), method: String::from("DELETE"), path: String::from("/home/main/jh/5654561654164-4654654654"), status_code: 404, response_size: 2649};
    //println!("{}", std::mem::size_of::log);

    // Allocations
    //let mut err: Vec<u16> = Vec::with_capacity(500);
    //let mut err = vec![100; 50000000];

    let mut statusCode = HashMap::new();
    // for i in 100..599 {
    // statusCode.insert(i, 0);
    // }

    let mut pathFrequency: HashMap<u16, HashMap<String, i32>> = HashMap::new();
    for i in 100..599 {
        let mut inner_map = HashMap::new();
        inner_map.insert(String::from(""), 0);
        pathFrequency.insert(i, inner_map);
    }

    // OPEN FILE
    let mut f = File::open("data/sample.log").unwrap();

    let linesAmount = BufReader::new(&f).lines().count();

    // Point the buffer back to the start
    f.seek(SeekFrom::Start(0));

    // Add a reader buffer
    let file = BufReader::new(f);

    for line in file.lines() {
        //TODO: FIX
        if let Ok(entry) = ApacheLogPaser::parse_line(line.unwrap()) {
            // println!("{:?}", entry);
            entries.push(entry);
        }
        //println!("{}", dt.to_rfc2822());
    }

    // TODO: FREQUENCY ANALYSIS

    // TODO: LOGLEVEL ERROR
    let mut errorCodes: Vec<usize> = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        // ERROR processing
        // if (400..=599).contains(&entry.status_code) {
        if entry.status_code <= 599 && entry.status_code >= 400 {
            errorCodes.push(index);
            *statusCode.entry(entry.status_code).or_insert(0) += 1;
            pathFrequency.entry(entry.status_code).and_modify(|e| {
                // e.entry(entry.path.clone()).and_modify(|v| *v += 1).or_insert(1);
                *e.entry(entry.path.clone()).or_insert(0) += 1;
                // println!("{}, {:?}", entry.status_code, e);
            });
        }
        //println!("{:?}", entry);
    }

    // println!("{:?}", errorCodes[11]);

    // for (key, value) in statusCode {
    // if value > 0 { println!("Status {} has a frequency of {}", key, value); }
    // }

    // Convert HashMap to vec (of pairs) and sort by value
    let mut sortedStatusCode: Vec<_> = statusCode.iter().collect();
    sortedStatusCode.sort_by(|a, b| b.1.cmp(a.1)); // Sort in descending order

    // Convert HashMap to vec (of pairs) and sort by value
    // let mut sortedPath: Vec<_> = pathFrequency.iter().collect();
    // sortedPath.sort_by(|a, b| (&(b.1).1).cmp(&(a.1).1));  // Sort in descending order

    let sortedPath: Vec<_> = pathFrequency
        .iter()
        .map(|(k, v)| {
            let mut sorted: Vec<_> = v.iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(a.1));
            (*k, sorted)
        })
        .collect();

    // Print sorted results
    // for (code, count) in sorted {
    // if *count > 0 {
    // println!("Status {} has a frequency of {}", code, count);
    // }
    // }

    let mut out = File::create("log/log.json").unwrap();
    let data = json!({
        "total_logs": linesAmount,
        "error_logs": errorCodes.len(),
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

    sys.refresh_all();
    println!("used memory : {} bytes", sys.used_memory());
}
