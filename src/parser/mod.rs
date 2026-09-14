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

        // Safe split_off wrapper
        let safe_split =
            |s: &mut String, idx: usize, err_msg: &str| -> Result<(), Box<dyn std::error::Error>> {
                if idx > s.len() || !s.is_char_boundary(idx) {
                    return Err(err_msg.into());
                }
                *s = s.split_off(idx);
                Ok(())
            };

        // Not efficient? ( O(n) )
        safe_split(
            &mut l,
            ip.len() + 6,
            "Invalid log format: ip section too short",
        )?;

        // Date operations
        let date = l.chars().take_while(|&c| c != ']').collect::<String>();
        safe_split(
            &mut l,
            date.len() + 3,
            "Invalid log format: date section too short",
        )?;
        let dt = DateTime::parse_from_str(&date, "%d/%b/%Y:%H:%M:%S %z")?;

        let args = l.chars().take_while(|&c| c != '"').collect::<String>();
        safe_split(
            &mut l,
            args.len() + 2,
            "Invalid log format: args section too short",
        )?;

        let args_segments: Vec<&str> = args.split(' ').collect();
        if args_segments.len() < 3 {
            return Err("Invalid log format: missing args segments".into());
        }

        // Status codes go from 100 to 599 (500 options)
        // 100 - 199 Info
        // 200 - 299 Success
        // 300 - 399 Redirection
        // 400 - 499 Client error
        // 500 - 599 Server error
        let status = l.chars().take_while(|&c| c != ' ').collect::<String>();
        safe_split(
            &mut l,
            status.len() + 1,
            "Invalid log format: status section too short",
        )?;

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
    fn test_parse_line_valid() {
        let log =
            "127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326";
        let res = ApacheLogPaser::parse_line(log.to_string());
        assert!(res.is_ok());
        let entry = res.unwrap();
        assert_eq!(entry.method, "GET");
        assert_eq!(entry.path, "/apache_pb.gif");
        assert_eq!(entry.protocol, "HTTP/1.0");
        assert_eq!(entry.status_code, 200);
        assert_eq!(entry.response_size, 2326);
    }

    #[test]
    fn test_parse_line_error_handling_ip_too_short() {
        let res = ApacheLogPaser::parse_line("bad".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Invalid log format: ip section too short"
        );
    }

    #[test]
    fn test_parse_line_error_handling_date_too_short() {
        let log = "127.0.0.1 - - [10/Oct/2000:13:55:36 -0700";
        let res = ApacheLogPaser::parse_line(log.to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Invalid log format: date section too short"
        );
    }

    #[test]
    fn test_parse_line_error_handling_args_too_short() {
        let log = "127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0";
        let res = ApacheLogPaser::parse_line(log.to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Invalid log format: args section too short"
        );
    }

    #[test]
    fn test_parse_line_error_handling_missing_args_segments() {
        let log = "127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] \"GET\" 200 2326";
        let res = ApacheLogPaser::parse_line(log.to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Invalid log format: missing args segments"
        );
    }

    #[test]
    fn test_parse_line_error_handling_status_too_short() {
        let log = "127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" ";
        let res = ApacheLogPaser::parse_line(log.to_string());
        assert!(res.is_err());
        // In this case `status` reads everything or empty, wait let's see.
        // `status` reads until ' '. The log ends with ' ', so status is empty string.
        // `l.split_off(status.len() + 1)` -> `split_off(0 + 1)`.
        // `l.len()` is 0. So it fails `idx > s.len()`!
        assert_eq!(
            res.unwrap_err().to_string(),
            "Invalid log format: status section too short"
        );
    }
}
