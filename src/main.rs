// TODO: ADD a formatter before processing the log file
// TODO: VALIDATE LOG
// TODO: ADD ERRORS

use parser::{ApacheLogPaser, LogParser};
use sysinfo::System;


use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::collections::HashMap;
use std::error::Error;


mod utils;
mod parser;


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
    for i in 100..599 {
       statusCode.insert(i, 0);
    }

    let mut f = File::open("data/sample.log").unwrap();
    

    let linesAmount = BufReader::new(&f).lines().count();

    // Point the buffer back to the start
    f.seek(SeekFrom::Start(0));

    // Add a reader buffer
    let file = BufReader::new(f); 

    for line in file.lines() {

        //TODO: FIX
        if let Ok(entry) = ApacheLogPaser::parse_line(line.unwrap()) {
            println!("{:?}", entry);
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
        }
        //println!("{:?}", entry);
    }

    // println!("{:?}", errorCodes[11]);
    
    // for (key, value) in statusCode {
        // if value > 0 { println!("Status {} has a frequency of {}", key, value); }
    // }

    // Convert HashMap to vec and sort by value
    let mut sorted: Vec<_> = statusCode.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));  // Sort in descending order
    
    // Print sorted results
    for (code, count) in sorted {
        if *count > 0 {
            println!("Status {} has a frequency of {}", code, count);
        }
    }

    sys.refresh_all();
    println!("used memory : {} bytes", sys.used_memory());

}
// TODO: SERIALIZE INTO JSON
