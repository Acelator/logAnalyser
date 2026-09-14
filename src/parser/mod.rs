use crate::utils::*;

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

        let ip = l.chars().take_while(|&c| c != ' ').collect::<String>();

        // Not efficient? ( O(n) )
        l = l.split_off(ip.len() + 6);

        // Date operations
        let date = l.chars().take_while(|&c| c != ']').collect::<String>();
        l = l.split_off(date.len() + 3);
        let dt = DateTime::parse_from_str(&date, "%d/%b/%Y:%H:%M:%S %z")?;

        let args = l.chars().take_while(|&c| c != '"').collect::<String>();
        l = l.split_off(args.len() + 2);

        let args_segments: Vec<&str> = args.split(' ').collect();

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
            ip: to_ip(ip),
            timestamp: dt.into(),

            method: args_segments[0].to_string(),
            path: args_segments[1].to_string(),
            protocol: args_segments[2].to_string(),

            status_code: status.parse::<u16>()?,
            response_size: size.parse::<usize>()?,
        };

        Ok(entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_apache_log_line() {
        let log_line =
            "127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326"
                .to_string();
        let result = ApacheLogPaser::parse_line(log_line);
        assert!(result.is_ok());
        let entry = result.unwrap();
        assert_eq!(entry.ip, [127, 0, 0, 1, 0, 0]);
        assert_eq!(entry.method, "GET");
        assert_eq!(entry.path, "/apache_pb.gif");
        assert_eq!(entry.protocol, "HTTP/1.0");
        assert_eq!(entry.status_code, 200);
        assert_eq!(entry.response_size, 2326);
    }

    #[test]
    fn test_invalid_status_code() {
        let log_line =
            "127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" ABC 2326"
                .to_string();
        let result = ApacheLogPaser::parse_line(log_line);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_date_format() {
        let log_line =
            "127.0.0.1 - - [10-Oct-2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326"
                .to_string();
        let result = ApacheLogPaser::parse_line(log_line);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_response_size() {
        let log_line =
            "127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 SIZE"
                .to_string();
        let result = ApacheLogPaser::parse_line(log_line);
        assert!(result.is_err());
    }
}
