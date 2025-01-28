// TODO: ADD a formatter before processing the log file
// TODO: VALIDATE LOG
// TODO: ADD ERRORS

use output::{JsonOutput, OutputData};
use hasher::md5;
use parser::{ApacheLogPaser, LogParser};

use sysinfo::System;

use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::Path;

mod hasher;
mod output;
mod parser;
mod utils;

fn main() {
    let mut sys = System::new_all();

    let mut hash = String::from("");
    let live_reload = true;
    // let log_level = ["INFO", "WARNING", "ERROR", "CRITICAL"];
    // let parser = ApacheLogPaser;
    // CHECKSUM

    if live_reload {
        // while true
        let mut i = 0;
        while i < 4 {
            // TODO: Create checkpoints (100mb for example and compute its checksum) so when comparing instead of doing it against all the file
            // TODO     it can be made on each partition. Also good potential for concurrency.
            let f_str = fs::read_to_string(Path::new("data/sample.log")).unwrap();
            let current_hash = md5(&f_str);
            println!("{}", current_hash);

            if current_hash != hash {
                println!("IM TIRED BOSS");
                mainLogic();
            } else {
                println!("ZZZZ MUCHO SUENO");
            }

            hash = current_hash;
            i += 1;
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    } else {
        mainLogic();
    }

    sys.refresh_all();
    println!("used memory : {} bytes", sys.used_memory());
}

fn mainLogic() {
    let mut entries: Vec<utils::LogEntry> = Vec::new();

    //let mut log = LogEntry {ip: [255,255,255,255,255,255], timestamp: Utc::now(), method: String::from("DELETE"), path: String::from("/home/main/jh/5654561654164-4654654654"), status_code: 404, response_size: 2649};
    //println!("{}", std::mem::size_of::log);

    // Allocations
    let mut status_code = HashMap::new();
    // for i in 100..599 {
    // statusCode.insert(i, 0);
    // }

    let mut path_frequency: HashMap<u16, HashMap<String, i32>> = HashMap::new();
    for i in 100..599 {
        let mut inner_map = HashMap::new();
        inner_map.insert(String::from(""), 0);
        path_frequency.insert(i, inner_map);
    }

    // OPEN FILE

    let mut f = File::open(Path::new("data/sample.log")).expect("Specified file doesn't exist");

    let lines_amount = BufReader::new(&f).lines().count();

    // Point the buffer back to the start
    f.seek(SeekFrom::Start(0));

    // Add a reader buffer
    let file = BufReader::new(f);

    for line in file.lines() {
        if let Ok(entry) = ApacheLogPaser::parse_line(line.unwrap_or_else(|e| {
            panic!(
                "Problem parsing the file with the specified parser, Error: {:?}",
                e
            );
        })) {
            // println!("{:?}", entry);
            entries.push(entry);
        }
        //println!("{}", dt.to_rfc2822());
    }

    // TODO: FREQUENCY ANALYSIS

    // TODO: LOGLEVEL ERROR
    let mut error_codes: Vec<usize> = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        // ERROR processing
        // if (400..=599).contains(&entry.status_code) {
        if entry.status_code <= 599 && entry.status_code >= 400 {
            error_codes.push(index);
            *status_code.entry(entry.status_code).or_insert(0) += 1;
            path_frequency.entry(entry.status_code).and_modify(|e| {
                // e.entry(entry.path.clone()).and_modify(|v| *v += 1).or_insert(1);
                *e.entry(entry.path.clone()).or_insert(0) += 1;
                // println!("{}, {:?}", entry.status_code, e);
            });
        }
        //println!("{:?}", entry);
    }

    // for (key, value) in statusCode {
    // if value > 0 { println!("Status {} has a frequency of {}", key, value); }
    // }

    // Convert HashMap to vec (of pairs) and sort by value
    let mut sorted_status_code: Vec<_> = status_code.iter().collect();
    sorted_status_code.sort_by(|a, b| b.1.cmp(a.1)); // Sort in descending order

    // Convert HashMap to vec (of pairs) and sort by value
    // let mut sortedPath: Vec<_> = pathFrequency.iter().collect();
    // sortedPath.sort_by(|a, b| (&(b.1).1).cmp(&(a.1).1));  // Sort in descending order

    let sorted_path: Vec<_> = path_frequency
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

    JsonOutput::output(
        lines_amount,
        &error_codes,
        &sorted_status_code,
        &sorted_path,
    );
}
