// TODO: ADD a formatter before processing the log file
// TODO: VALIDATE LOG
// TODO: ADD ERRORS

use chrono::prelude::*;
use sysinfo::{System};


use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::collections::HashMap;
use std::error::Error;



// Determine memory footprint
#[derive(Debug)]
struct LogEntry {
    // Ip in Ipv6 format (Using net Ipadrr?)
    ip: [u16;6],
    timestamp: chrono::DateTime<chrono::Utc>,
    method: String,
    path: String,
    status_code: u16,
    response_size: usize,
}


// Apache log format

// String or &str?
fn parse_line(line: String) -> Result<LogEntry, Box<dyn std::error::Error>> {
    let mut l = line;

    // TODO: CONVERT TO NUMBERS
    let ip = l.chars().take_while(|&c| c != ' ').collect::<String>();

    // Not efficient? ( O(n) )
    l = l.split_off(ip.len() + 6);


    // Date operations 
    let mut date = l.chars().take_while(|&c| c != ']').collect::<String>();
    l = l.split_off(date.len() + 3);
    let dt = DateTime::parse_from_str(&date, "%d/%b/%Y:%H:%M:%S %z").unwrap();


    let args = l.chars().take_while(|&c| c != '"').collect::<String>();
    l = l.split_off(args.len() + 2);


    // Status codes go from 100 to 599 (500 options)
    // 100 - 199 Info
    // 200 - 299 Success
    // 300 - 399 Redirection
    // 400 - 499 Client error
    // 500 - 599 Server error
    let status = l.chars().take_while(|&c| c != ' ').collect::<String>();
    l = l.split_off(status.len() + 1);

    // Update hashmap of statusCodes to allow for frequency analysis
    //*statusCode.entry(status).or_insert(0) += 1;


    let mut size = l;


    let entry = LogEntry {
        ip: [0,0,0,0,0,0],
        timestamp: dt.into(),

        // TODO: EXTRACT INFO FROM ARGS
        method: String::from("GET"),
        path: String::from("/"),

        status_code: status.parse::<u16>().unwrap(),
        response_size: size.parse::<usize>().unwrap(), 
    };

    Ok(entry)
}


fn main() {
    let mut sys = System::new_all();


    let mut entries: Vec<LogEntry> = Vec::new();

    //let mut log = LogEntry {ip: [255,255,255,255,255,255], timestamp: Utc::now(), method: String::from("DELETE"), path: String::from("/home/main/jh/5654561654164-4654654654"), status_code: 404, response_size: 2649};
    //println!("{}", std::mem::size_of::log);


    // Allocations
    //let mut err: Vec<u16> = Vec::with_capacity(500);
    //let mut err = vec![100; 50000000];
    //let mut statusCode = HashMap::new();
    //for i in 100..599 {
    //    statusCode.insert(i, 0);
    //}

    let mut f = File::open("data/sample.log").unwrap();
    

    let linesAmount = BufReader::new(&f).lines().count();

    // Point the buffer back to the start
    f.seek(SeekFrom::Start(0));

    // Add a reader buffer
    let file = BufReader::new(f); 

    for line in file.lines() {

        if let Ok(entry) = parse_line(line.unwrap()) {
            println!("{:?}", entry);
            entries.push(entry);
        }



        //println!("{}", dt.to_rfc2822());

    }

    // println!("{:?}", entries[19999]);
    
    //for (key, value) in statusCode {
    //    println!("Status {} has a frequency of {}", key, value);
    //}


    sys.refresh_all();
    println!("used memory : {} bytes", sys.used_memory());

}
