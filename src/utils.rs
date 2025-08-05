use std::fs::{metadata, File};
use std::io::{Read, Seek, SeekFrom};

use std::path::PathBuf;

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

pub fn compute_hash(path: &std::path::Path, mb: u32, hash: &mut Vec<String>) -> String {
    let mut f = File::open(path).expect("File doesn't exist");
    let file_size = metadata(path).unwrap().len();
    
    let chunk_size = 2_u64.pow(mb);
    let partitions: u64 = std::cmp::max(file_size.div_ceil(chunk_size), 1);

    for i in 0..partitions {
        let offset = chunk_size * i;
        f.seek(SeekFrom::Start(offset)).unwrap();

        // Calculate actual bytes to read (handle end of file)
        let bytes_to_read = std::cmp::min(chunk_size, file_size - offset);
        let mut buf = vec![0u8; bytes_to_read as usize];
        
        // Use read instead of read_exact to handle partial reads at end of file
        f.read_exact(&mut buf).unwrap_or_else(|_| {
            // If read_exact fails, try reading what's available
            f.seek(SeekFrom::Start(offset)).unwrap();
            let bytes_read = f.read(&mut buf).unwrap();
            buf.truncate(bytes_read);
        });

        let current_hash = md5_bits(&mut buf);

        if i < hash.len() as u64 && current_hash != hash[i as usize] {
            // Recompute all remaining hashes from this point
            for j in i..partitions {
                let offset_j = chunk_size * j;
                f.seek(SeekFrom::Start(offset_j)).unwrap();
                
                let bytes_to_read_j = std::cmp::min(chunk_size, file_size - offset_j);
                let mut buf_j = vec![0u8; bytes_to_read_j as usize];
                
                f.read_exact(&mut buf_j).unwrap_or_else(|_| {
                    f.seek(SeekFrom::Start(offset_j)).unwrap();
                    let bytes_read = f.read(&mut buf_j).unwrap();
                    buf_j.truncate(bytes_read);
                });

                let hash_j = md5_bits(&mut buf_j);
                if j < hash.len() as u64 {
                    hash[j as usize] = hash_j;
                }
            }
            break;
        } else if i < hash.len() as u64 {
            hash[i as usize] = current_hash;
        }
    }

    // Concatenate all hash strings
    hash.iter().filter(|h| !h.is_empty()).cloned().collect::<String>()
}

#[derive(Debug)]
pub struct Hash {
    #[allow(dead_code)]
    pub path: PathBuf,
    pub hash: String,
}

#[derive(Parser, Debug)]
#[command()]
pub struct Config {
    #[arg(short, long)]
    pub path: String,

    #[arg(short, long)]
    pub live_reload: bool,
}
