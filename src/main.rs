// TODO: ADD a formatter before processing the log file
// TODO: VALIDATE LOG
// TODO: ADD ERRORS

use output::{DatabaseOutput, OutputData};
use parser::{ApacheLogParser, LogParser};

use sysinfo::System;

use rusqlite::{params, Connection, Result};
use utils::{compute_hash, Config};
use clap::Parser;
use rayon::prelude::*;

use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;
use std::sync::Mutex;

// Configuration constants
const DEV_MODE: bool = true;
const DATABASE_PATH: &str = "db/main.db";
const CHUNK_SIZE_MB: u32 = 20;

mod hasher;
mod output;
mod parser;
mod utils;

fn main() -> Result<()> {
    let mut sys = System::new_all();
    let mut conn = Connection::open(DATABASE_PATH)?;

    initialize_database(&mut conn)?;
    let config = Config::parse();
    println!("CONFIG: {:?}", config);

    let log_file_path = Path::new(&config.path);
    let (_partitions, hashes) = setup_file_processing(log_file_path)?;

    initialize_hash_table(&mut conn)?;

    if config.live_reload {
        run_live_reload_loop(&mut conn, log_file_path, &mut hashes.clone())?;
    } else {
        main_logic(log_file_path);
    }

    conn.close().expect("Error closing database");
    
    sys.refresh_all();
    println!("Used memory: {} bytes", sys.used_memory());

    Ok(())
}

fn initialize_database(conn: &mut Connection) -> Result<()> {
    if DEV_MODE {
        let _ = conn.execute("DROP TABLE IF EXISTS logs;", []);
    }

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
    )?;
    
    Ok(())
}

fn setup_file_processing(log_file_path: &Path) -> Result<(u64, Vec<String>)> {
    let metadata = fs::metadata(log_file_path).expect("No metadata on file");
    let partitions: u64 = std::cmp::max(metadata.len().div_ceil(2_u64.pow(CHUNK_SIZE_MB)), 1);
    
    println!("Partitions required: {}", partitions);
    
    let hashes = vec![String::new(); partitions as usize];
    Ok((partitions, hashes))
}

fn initialize_hash_table(conn: &mut Connection) -> Result<()> {
    if DEV_MODE {
        let _ = conn.execute("DROP TABLE IF EXISTS hash;", []);
    }

    conn.execute(
        "CREATE TABLE IF NOT EXISTS hash (
            id INTEGER PRIMARY KEY,
            path TEXT NOT NULL,
            hash TEXT NOT NULL
        )",
        [],
    )?;
    
    Ok(())
}

fn run_live_reload_loop(conn: &mut Connection, log_file_path: &Path, hashes: &mut Vec<String>) -> Result<()> {
    for iteration in 0..4 {
        println!("Starting main loop iteration: {}", iteration + 1);

        let current_hash = get_stored_hash(conn, log_file_path)?;
        let new_hash = compute_hash(log_file_path, CHUNK_SIZE_MB, hashes);

        if new_hash != current_hash {
            store_hash(conn, log_file_path, &new_hash)?;
            println!("File changed, processing...");
            main_logic(log_file_path);
        } else {
            println!("No changes detected");
        }

        std::thread::sleep(std::time::Duration::from_secs(5));
    }
    
    Ok(())
}

fn get_stored_hash(conn: &mut Connection, log_file_path: &Path) -> Result<String> {
    let path_str = log_file_path.to_str().unwrap();
    
    match conn.query_row("SELECT hash FROM hash WHERE path = ?1", params![path_str], |row| {
        Ok(row.get::<_, String>(0)?)
    }) {
        Ok(hash) => Ok(hash),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(String::new()),
        Err(e) => Err(e),
    }
}

fn store_hash(conn: &mut Connection, log_file_path: &Path, hash: &str) -> Result<()> {
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT OR REPLACE INTO hash (path, hash) VALUES (?1, ?2)",
        params![log_file_path.to_str().unwrap(), hash],
    )?;
    tx.commit()?;
    Ok(())
}

fn main_logic(log_file_path: &Path) {
    // OPEN FILE
    let mut f = File::open(log_file_path).expect("Specified file doesn't exist");

    let lines_amount = BufReader::new(&f).lines().count();

    // Point the buffer back to the start
    let _ = f.seek(SeekFrom::Start(0));

    // Add a reader buffer and collect all lines first
    let file = BufReader::new(f);
    let lines: Vec<String> = file.lines().collect::<Result<Vec<_>, _>>().unwrap_or_else(|e| {
        panic!("Problem reading file lines, Error: {:?}", e);
    });

    // Parse log entries in parallel
    let entries: Vec<utils::LogEntry> = lines
        .par_iter()
        .filter_map(|line| {
            ApacheLogParser::parse_line(line.clone()).ok()
        })
        .collect();

    // Process error analysis in parallel
    let status_code = Mutex::new(HashMap::new());
    let path_frequency = Mutex::new({
        let mut pf = HashMap::new();
        for i in 100..599 {
            let mut inner_map = HashMap::new();
            inner_map.insert(String::from(""), 0);
            pf.insert(i, inner_map);
        }
        pf
    });

    let error_codes: Vec<usize> = entries
        .par_iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            if entry.status_code >= 400 && entry.status_code <= 599 {
                // Update status code count
                {
                    let mut sc = status_code.lock().unwrap();
                    *sc.entry(entry.status_code).or_insert(0) += 1;
                }
                
                // Update path frequency
                {
                    let mut pf = path_frequency.lock().unwrap();
                    pf.entry(entry.status_code).and_modify(|e| {
                        *e.entry(entry.path.clone()).or_insert(0) += 1;
                    });
                }
                
                Some(index)
            } else {
                None
            }
        })
        .collect();

    let status_code = status_code.into_inner().unwrap();
    let path_frequency = path_frequency.into_inner().unwrap();

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

    let conn = Connection::open(DATABASE_PATH).expect("Failed to open database");
    DatabaseOutput::output(
        lines_amount,
        &error_codes,
        &sorted_status_code,
        &sorted_path,
        &entries,
        Some(conn),
    );
}
