// TODO: ADD a formatter before processing the log file
// TODO: VALIDATE LOG
// TODO: ADD ERRORS

use hasher::md5_bits;
use output::{DatabaseOutput, JsonOutput, OutputData};
use parser::{ApacheLogPaser, LogParser};

use sysinfo::System;

use rusqlite::{params, Connection, Result, Rows};
use utils::{compute_hash, hash};

use core::error;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

mod hasher;
mod output;
mod parser;
mod utils;

const DEV: bool = true;

fn main() -> Result<()> {
    let mut sys = System::new_all();

    let mut conn = Connection::open("db/main.db")?;

    if DEV {
        let _ = conn.execute("DROP table logs;", []);
    }

    // PREPARATE DB CONNECTION, IN FUTURE CHANGE IT
    //TODO? change ip to number?
    conn.execute(
        "CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY,
            ip TEXT NOT NULL,
            -- timestamp DATETIME NOT NULL,
            method TEXT NOT NULL,
            path TEXT NOT NULL,
            status_code INTEGER NOT NULL,
            response_size INTEGER NOT NULL
        )",
        [],
    )?;

    // Config
    // TODO: ALLOW CLI TO CHANGE VALUES

    let log_file_path = Path::new("data/sample.log");
    let live_reload = true;
    // let log_level = ["INFO", "WARNING", "ERROR", "CRITICAL"];
    // let parser = ApacheLogPaser;
    // let output_mode = JsonOutput;

    //* Main program  */
    let metadata = fs::metadata(log_file_path).expect("No metadata on file");

    let mb = 20; // Numbers of partitions needed to split the file with 200mb each one
    let partitions: u64 = std::cmp::max(metadata.len().div_ceil(2_u64.pow(mb)) - 1, 1);
    println!("Partitions required are {}", partitions);

    let mut hashes: Vec<String> = Vec::with_capacity(partitions as usize);

    for _i in 0..partitions {
        hashes.push(String::from(""));
    }

    if DEV {
        let _ = conn.execute("DROP table hash;", []);
    }

    // Each MD5 hash is 32 characters long
    // ! Cant return id
    conn.execute(
        "CREATE TABLE IF NOT EXISTS hash (
            id INTEGER PRIMARY KEY,
            path TEXT NOT NULL,
            hash TEXT NOT NULL
        )",
        [],
    )?;

    // CHECKSUM
    if live_reload {
        // while true
        let mut i = 0;
        while i < 4 {
            println!("STARTING MAIN LOOP");

            let mut current_hash = String::from("");

            let p = log_file_path.to_str().unwrap();
            println!("PATH IS {}", p);

            let a: Result<hash> =
                conn.query_row("SELECT * FROM hash WHERE path = ?1", params![p], |row| {
                    println!("/ROW: {:?}", row.get::<_, String>(1)?);
                    Ok(hash {
                        path: PathBuf::from(row.get::<_, String>(1)?),
                        hash: row.get(2)?,
                    })
                });
            // let a = conn.query_row("SELECT * FROM hash WHERE path = p")?;

            match a {
                Ok(hash) => {
                    println!(" CURRENT HASH");
                    println!("{:?}", hash);

                    // TODO: CHANGE NAMES
                    current_hash = hash.hash;
                }

                // We need to check if the problem is that there isnt a record associated to that file
                //  or if it is another err.
                Err(e) => match e {
                    rusqlite::Error::QueryReturnedNoRows => {}
                    _other => panic!("HELPDAS"),
                },
            }

            // TODO! Check if hash already exists in db, in that case it should be passed as "hashes"
            // TODO! Also if it hasn't been changed it shouldn't be store again
            let hash_str = compute_hash(log_file_path, mb, &mut hashes);
            println!("hashstr {:?}", hash_str);

            if hash_str == current_hash {
                println!("NOT WORKING");
            } else {
                // SAVE TO DB
                {
                    let tx = conn.transaction().unwrap();
                    tx.execute(
                        "INSERT OR REPLACE INTO hash (path, hash)  VALUES (?1, ?2)",
                        params![log_file_path.to_str().unwrap(), hash_str],
                    )
                    .expect("ERROR STORING HASH");
                    tx.commit().expect("Failed to commit transaction");
                }

                println!("{:?}", hashes);

                println!("WORKING");
                mainLogic(log_file_path);
            }

            i += 1;
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    } else {
        mainLogic(log_file_path);
    }

    conn.close().expect("Err closing db");

    sys.refresh_all();
    println!("used memory : {} bytes", sys.used_memory());

    Ok(())
}

fn mainLogic(log_file_path: &Path) {
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

    let mut f = File::open(log_file_path).expect("Specified file doesn't exist");

    let lines_amount = BufReader::new(&f).lines().count();

    // Point the buffer back to the start
    let _ = f.seek(SeekFrom::Start(0));

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

    // JsonOutput::output(
    //     lines_amount,
    //     &error_codes,
    //     &sorted_status_code,
    //     &sorted_path,
    //     None
    // );

    let conn = Connection::open("db/main.db").expect("msg");
    DatabaseOutput::output(
        lines_amount,
        &error_codes,
        &sorted_status_code,
        &sorted_path,
        &entries,
        Option::Some(conn),
    );
}
