use std::fs::{metadata, File};
use std::io::{Read, Seek, SeekFrom};

use std::path::{Path, PathBuf};

use clap::{command, Parser};
use serde::{Deserialize, Serialize};

use crate::hasher::md5_bits;

// Determine memory footprint
#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    // Ip in Ipv6 format (Using net IpAdrr?)
    pub ip: [u16; 6],
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub method: String,
    pub path: String,
    pub protocol: String,
    pub status_code: u16,
    pub response_size: usize,
}

// OPTIMIZAR -> >10% of time spent here
pub fn to_ip(l: String) -> [u16; 6] {
    let mut ip: [u16; 6] = [0, 0, 0, 0, 0, 0];

    let segments: Vec<&str> = l.split('.').collect();

    // Convert each segment to u16, up to 6 segments
    for (i, segment) in segments.iter().enumerate() {
        if i >= 6 {
            break;
        } // Don't exceed array bounds

        ip[i] = segment.parse::<u16>().unwrap_or(0);
    }

    ip
}

// fn seek_read(mut reader: impl Read + Seek, offset: u64, buf: &mut [u8]) -> io::Result<()> {
//     reader.seek(SeekFrom::Start(offset))?;
//     reader.read_exact(buf)?;
//     Ok(())
// }

pub fn compute_hash(path: &std::path::Path, mb: u32, mut hash: &mut Vec<String>) -> String {
    let mut f = File::open(path).expect("File doesn't exist");

    let partitions: u64 =
        std::cmp::max(metadata(path).unwrap().len().div_ceil(2_u64.pow(mb)) - 1, 1);

    for _i in 0..partitions {
        f.seek(SeekFrom::Start((2_i32.pow(mb) as u64 * _i)))
            .unwrap();

        let mut buf = vec![0u8; 2_u64.pow(mb) as usize];
        f.read_exact(&mut buf).unwrap();

        let current_hash_i = md5_bits(&mut buf);
        println!("size {}", std::mem::size_of_val(&current_hash_i));

        if current_hash_i != hash[_i as usize] {
            println!("IM TIRED BOSS");
            for j in _i..partitions {
                f.seek(SeekFrom::Start(2_i32.pow(mb) as u64 * j)).unwrap();

                let mut buf = vec![0u8; 2_u64.pow(mb) as usize];
                f.read_exact(&mut buf).unwrap();

                let current_hash_j = md5_bits(&mut buf);
                hash[j as usize] = current_hash_j;
            }
            break;
        } else {
            hash[_i as usize] = current_hash_i;
            println!("ZZZZ MUCHO SUENO");
        }
    }

    let mut hash_str = String::new();
    for hash in hash {
        hash_str.push_str(hash);
    }

    hash_str
}

#[derive(Debug)]
pub struct hash {
    pub path: PathBuf,
    pub hash: String,
}

#[derive(Parser, Debug)]
#[command()]
pub struct Config {
    #[arg(short, long)]
    pub path: String,

    #[arg(short, long, action)]
    pub live_reload: bool,
}
