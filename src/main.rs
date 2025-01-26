// TODO: ADD a formatter before processing the log file

use chrono::prelude::*;
use sysinfo::{Component, System};


use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};




struct LogEntry {
    ip: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    method: String,
    path: String,
    status_code: u16,
    response_size: usize,
}

fn main() {
    let mut sys = System::new_all();


    // Allocations
    let mut err: Vec<u16> = Vec::with_capacity(500);
    //let mut err = vec![100; 50000000];

    let mut f = File::open("data/sample.log").unwrap();
    

    let linesAmount = BufReader::new(&f).lines().count();

    // Point the buffer back to the start
    f.seek(SeekFrom::Start(0));

    // Add a reader buffer
    let mut file = BufReader::new(f); 

    for line in file.lines() {
        let mut l = line.unwrap();
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
        let status = l.chars().take_while(|&c| c != ' ').collect::<String>();
        l = l.split_off(status.len() + 2);



        let mut size = l;




        //println!("{}", dt.to_rfc2822());



    }



    sys.refresh_all();
    println!("used memory : {} bytes", sys.used_memory());

}
