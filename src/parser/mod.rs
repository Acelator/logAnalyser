use crate::utils::{self, LogEntry};

use chrono::prelude::*;

pub trait LogParser {
    fn parse_line(line: String) -> Result<LogEntry, Box<dyn std::error::Error>>;
}

// Common log format
// Structure -> host ident authuser date request status bytes

pub struct ApacheLogPaser;

impl LogParser for ApacheLogPaser {
    // String or &str?
    fn parse_line(line: String) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let mut l = line;

        // TODO: CONVERT TO NUMBERS
        let ip = l.chars().take_while(|&c| c != ' ').collect::<String>();

        // Not efficient? ( O(n) )
        l = l.split_off(ip.len() + 6);

        // Date operations
        let date = l.chars().take_while(|&c| c != ']').collect::<String>();
        l = l.split_off(date.len() + 3);
        let dt = DateTime::parse_from_str(&date, "%d/%b/%Y:%H:%M:%S %z").unwrap();

        let args = l.chars().take_while(|&c| c != '"').collect::<String>();
        l = l.split_off(args.len() + 2);

        let argsSegments: Vec<&str> = args.split(' ').collect();

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

        let size = l;

        // Everything is parsed, now we can create the LogEntry
        let entry = LogEntry {
            ip: utils::toIp(ip),
            timestamp: dt.into(),

            // TODO: EXTRACT INFO FROM ARGS
            method: argsSegments[0].to_string(),
            path: argsSegments[1].to_string(),
            protocol: argsSegments[2].to_string(),

            status_code: status.parse::<u16>().unwrap(),
            response_size: size.parse::<usize>().unwrap(),
        };

        Ok(entry)
    }
}
