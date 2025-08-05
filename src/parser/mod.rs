use crate::utils::{LogEntry, to_ip};
use chrono::prelude::*;

pub trait LogParser {
    fn parse_line(line: String) -> Result<LogEntry, Box<dyn std::error::Error>>;
}

/// Apache Common Log Format parser
/// Format: host ident authuser date request status bytes
/// Example: 127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] "GET /apache_pb.gif HTTP/1.0" 200 2326
pub struct ApacheLogParser;

impl LogParser for ApacheLogParser {
    fn parse_line(line: String) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let mut remaining = line;

        // Extract IP address
        let ip_str = extract_until_char(&mut remaining, ' ')?;
        skip_chars(&mut remaining, 6); // Skip " - - ["

        // Extract timestamp
        let date_str = extract_until_char(&mut remaining, ']')?;
        skip_chars(&mut remaining, 3); // Skip "] \""
        let timestamp = DateTime::parse_from_str(&date_str, "%d/%b/%Y:%H:%M:%S %z")?;

        // Extract request line (method, path, protocol)
        let request_str = extract_until_char(&mut remaining, '"')?;
        skip_chars(&mut remaining, 2); // Skip "\" "
        let request_parts: Vec<&str> = request_str.split(' ').collect();
        
        if request_parts.len() < 3 {
            return Err("Invalid request format".into());
        }

        // Extract status code
        let status_str = extract_until_char(&mut remaining, ' ')?;
        skip_chars(&mut remaining, 1); // Skip space

        // Remaining is response size
        let response_size_str = remaining.trim();

        let entry = LogEntry {
            ip: to_ip(ip_str),
            timestamp: timestamp.into(),
            method: request_parts[0].to_string(),
            path: request_parts[1].to_string(),
            protocol: request_parts[2].to_string(),
            status_code: status_str.parse::<u16>()?,
            response_size: response_size_str.parse::<usize>()?,
        };

        Ok(entry)
    }
}

/// Extract characters from the beginning of a string until a specific character is found
fn extract_until_char(input: &mut String, delimiter: char) -> Result<String, Box<dyn std::error::Error>> {
    let pos = input.find(delimiter).ok_or("Delimiter not found")?;
    let extracted = input.chars().take(pos).collect::<String>();
    *input = input.split_off(pos);
    Ok(extracted)
}

/// Skip a specified number of characters from the beginning of a string
fn skip_chars(input: &mut String, count: usize) {
    if input.len() >= count {
        *input = input.split_off(count);
    }
}
