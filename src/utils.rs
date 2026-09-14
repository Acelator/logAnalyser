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

pub fn compute_hash(
    path: &std::path::Path,
    mb: u32,
    hash: &mut Vec<String>,
) -> std::io::Result<String> {
    let mut f = File::open(path)?;

    let partitions: u64 = std::cmp::max(metadata(path)?.len().div_ceil(2_u64.pow(mb)) - 1, 1);

    for _i in 0..partitions {
        f.seek(SeekFrom::Start(2_i32.pow(mb) as u64 * _i))?;

        let mut buf = vec![0u8; 2_u64.pow(mb) as usize];
        f.read_exact(&mut buf)?;

        let current_hash_i = md5_bits(&mut buf);
        println!("size {}", std::mem::size_of_val(&current_hash_i));

        if current_hash_i != hash[_i as usize] {
            println!("IM TIRED BOSS");
            for j in _i..partitions {
                f.seek(SeekFrom::Start(2_i32.pow(mb) as u64 * j))?;

                let mut buf = vec![0u8; 2_u64.pow(mb) as usize];
                f.read_exact(&mut buf)?;

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

    Ok(hash_str)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_ip_happy_path() {
        assert_eq!(to_ip("192.168.1.1".to_string()), [192, 168, 1, 1, 0, 0]);
        assert_eq!(to_ip("10.0.0.1.2.3".to_string()), [10, 0, 0, 1, 2, 3]);
    }

    #[test]
    fn test_to_ip_more_than_6_segments() {
        assert_eq!(to_ip("1.2.3.4.5.6.7.8".to_string()), [1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_to_ip_non_numeric_segments() {
        assert_eq!(to_ip("192.168.abc.1".to_string()), [192, 168, 0, 1, 0, 0]);
        assert_eq!(to_ip("foo.bar.baz".to_string()), [0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_to_ip_empty_string() {
        // "".split('.') yields one empty string segment `[""]`
        // "".parse::<u16>() is an error, so it returns 0.
        assert_eq!(to_ip("".to_string()), [0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_to_ip_out_of_bounds_u16() {
        // 70000 exceeds u16::MAX (65535)
        assert_eq!(to_ip("70000.1.2.3".to_string()), [0, 1, 2, 3, 0, 0]);
    }
}
