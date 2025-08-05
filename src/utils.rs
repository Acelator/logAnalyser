use std::fs::{metadata, File};
use std::io::{Read, Seek, SeekFrom};

use clap::{command, Parser};
use serde::{Deserialize, Serialize};

use crate::hasher::md5_bits;

/// Represents a parsed log entry with structured data
#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    /// IP address in simplified format (up to 6 segments)
    pub ip: [u16; 6],
    /// Timestamp of the request
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Requested path/URL
    pub path: String,
    /// HTTP protocol version
    pub protocol: String,
    /// HTTP status code
    pub status_code: u16,
    /// Response size in bytes
    pub response_size: usize,
}

/// Parse IP address string into array format
/// Converts IP segments to u16 values, supporting up to 6 segments
pub fn to_ip(ip_str: String) -> [u16; 6] {
    let mut ip: [u16; 6] = [0, 0, 0, 0, 0, 0];
    let segments: Vec<&str> = ip_str.split('.').collect();

    for (i, segment) in segments.iter().enumerate() {
        if i >= 6 {
            break;
        }
        ip[i] = segment.parse::<u16>().unwrap_or(0);
    }

    ip
}

// fn seek_read(mut reader: impl Read + Seek, offset: u64, buf: &mut [u8]) -> io::Result<()> {
//     reader.seek(SeekFrom::Start(offset))?;
//     reader.read_exact(buf)?;
//     Ok(())
// }

/// Compute MD5 hash for a file in chunks
/// 
/// This function reads a file in chunks and computes MD5 hashes for each chunk.
/// It can efficiently detect if only part of the file has changed by comparing
/// hashes chunk by chunk and only recomputing changed portions.
///
/// # Arguments
/// * `path` - Path to the file to hash
/// * `mb` - Power of 2 for chunk size (e.g., 20 for 1MB chunks)
/// * `hash` - Mutable vector to store computed hashes
///
/// # Returns
/// Concatenated string of all chunk hashes
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

/// Command line configuration
#[derive(Parser, Debug)]
#[command()]
pub struct Config {
    /// Path to the log file to analyze
    #[arg(short, long)]
    pub path: String,

    /// Enable live reload mode (monitors file for changes)
    #[arg(short, long)]
    pub live_reload: bool,
}
