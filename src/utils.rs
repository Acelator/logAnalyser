use serde::{Serialize, Deserialize};

// Determine memory footprint
#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    // Ip in Ipv6 format (Using net IpAdrr?)
    pub ip: [u16;6],
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub method: String,
    pub path: String,
    pub protocol: String,
    pub status_code: u16,
    pub response_size: usize,
}


// OPTIMIZAR -> >10% of time spent here
pub fn toIp(l: String) -> [u16; 6] {
    let mut ip: [u16;6] = [0,0,0,0,0,0];

    let segments: Vec<&str> = l.split('.').collect();

    // Convert each segment to u16, up to 6 segments
    for (i, segment) in segments.iter().enumerate() {
        if i >= 6 { break; }  // Don't exceed array bounds
        
        ip[i] = segment.parse::<u16>().unwrap_or(0);
    }
    

    ip
}