use crate::utils::*;

use chrono::prelude::*;

pub trait LogParser {
    fn parse_line(line: &str) -> Result<LogEntry, Box<dyn std::error::Error>>;
}

// Common log format
// Structure -> host ident authuser date request status bytes

pub struct ApacheLogPaser;

impl LogParser for ApacheLogPaser {
    fn parse_line(line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let ip_end = line.find(' ').ok_or("Invalid format: no IP")?;
        let ip = &line[..ip_end];

        let date_start = line.find('[').ok_or("Invalid format: no date start")? + 1;
        let date_end = line[date_start..]
            .find(']')
            .ok_or("Invalid format: no date end")?
            + date_start;
        let date = &line[date_start..date_end];
        let dt = DateTime::parse_from_str(date, "%d/%b/%Y:%H:%M:%S %z")?;

        let args_start = line.find('"').ok_or("Invalid format: no args start")? + 1;
        let args_end = line[args_start..]
            .find('"')
            .ok_or("Invalid format: no args end")?
            + args_start;
        let args = &line[args_start..args_end];

        let mut args_split = args.split(' ');
        let method = args_split.next().unwrap_or("").to_string();
        let path = args_split.next().unwrap_or("").to_string();
        let protocol = args_split.next().unwrap_or("").to_string();

        let status_start = args_end + 2;
        let status_end = line[status_start..]
            .find(' ')
            .ok_or("Invalid format: no status")?
            + status_start;
        let status = &line[status_start..status_end];

        let size = &line[status_end + 1..];

        // Everything is parsed, now we can create the LogEntry
        let entry = LogEntry {
            ip: to_ip(ip),
            timestamp: dt.into(),

            method,
            path,
            protocol,

            status_code: status.parse::<u16>()?,
            response_size: size.parse::<usize>().unwrap_or(0),
        };

        Ok(entry)
    }
}
