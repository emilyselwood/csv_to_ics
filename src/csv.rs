use std::{
    fs::File,
    io::{self, Read as _},
};

pub struct CSVData {
    pub headers: Vec<String>,
    pub data: Vec<Vec<String>>,
}

enum CSVState {
    InSeparator,
    InRecord,
    InQuotes,
}

const CSV_SEPARATOR: u8 = b',';
const CSV_QUOTE: u8 = b'"';
const CSV_NEWLINE: u8 = b'\n';

pub fn parse_csv_file(path: &str) -> Result<CSVData, io::Error> {
    let mut f = File::open(path)?;

    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;

    Ok(parse_csv(buffer.as_slice()))
}

pub fn parse_csv_string(data: &str) -> CSVData {
    parse_csv(data.as_bytes())
}

pub fn parse_csv(buffer: &[u8]) -> CSVData {
    let mut data = Vec::new();
    let mut headers = Vec::new();

    let mut i = 0;
    let mut state = CSVState::InSeparator;
    let mut in_header: bool = true;
    let mut record_buffer = "".to_owned();
    let mut line_buffer: Vec<String> = Vec::new();

    while i < buffer.len() {
        match state {
            CSVState::InSeparator => {
                if !buffer[i].is_ascii_whitespace() || buffer[i] == CSV_NEWLINE {
                    state = CSVState::InRecord
                } else {
                    i += 1;
                }
            }
            CSVState::InRecord => {
                if buffer[i] == CSV_SEPARATOR {
                    if !record_buffer.is_empty() {
                        line_buffer.push(record_buffer);
                        record_buffer = "".to_owned();
                    }
                    state = CSVState::InSeparator;
                    i += 1;
                } else if buffer[i] == CSV_QUOTE {
                    state = CSVState::InQuotes;
                    i += 1;
                } else if buffer[i] == CSV_NEWLINE {
                    if !record_buffer.is_empty() {
                        line_buffer.push(record_buffer.clone());
                        record_buffer = "".to_owned();
                    }

                    if !line_buffer.is_empty() {
                        if in_header {
                            headers = line_buffer.clone();
                            in_header = false;
                        } else {
                            data.push(line_buffer.clone());
                        }
                        line_buffer = Vec::new();
                    }

                    i += 1;
                } else {
                    record_buffer.push(buffer[i].into());
                    i += 1;
                }
            }
            CSVState::InQuotes => {
                if buffer[i] == CSV_QUOTE {
                    state = CSVState::InRecord;
                    i += 1;
                } else {
                    record_buffer.push(buffer[i].into());
                    i += 1;
                }
            }
        }
    }
    if !record_buffer.is_empty() {
        line_buffer.push(record_buffer.clone());
    }
    if !line_buffer.is_empty() {
        data.push(line_buffer);
    }

    let result = CSVData {
        headers: headers,
        data: data,
    };

    result
}

#[cfg(test)]
mod tests {
    use crate::csv::parse_csv_string;

    #[test]
    fn test_basic_csv_parsing() {
        let input = "c_a, c_b, c_c, c_d\n1,2,3,4\n5, 6, 7, 8";

        let result = parse_csv_string(input);

        assert_eq!(result.headers.as_slice(), &["c_a", "c_b", "c_c", "c_d"]);

        assert_eq!(result.data.len(), 2);

        assert_eq!(result.data[0].as_slice(), &["1", "2", "3", "4"]);
        assert_eq!(result.data[1].as_slice(), &["5", "6", "7", "8"]);
    }

    #[test]
    fn test_quoted_csv_parsing() {
        let input = "c_a, c_b, c_c, c_d\n1,2,\"3, wibble\",4\n5, 6, \"7\", 8";
        let result = parse_csv_string(input);

        assert_eq!(result.headers.as_slice(), &["c_a", "c_b", "c_c", "c_d"]);

        assert_eq!(result.data.len(), 2);

        assert_eq!(result.data[0].as_slice(), &["1", "2", "3, wibble", "4"]);
        assert_eq!(result.data[1].as_slice(), &["5", "6", "7", "8"]);
    }

    #[test]
    fn test_empty_final_values() {
        let input = "c_a, c_b, c_c, c_d\n1,2,\"3, wibble\",4,\n5, 6, \"7\", 8\n";
        let result = parse_csv_string(input);

        assert_eq!(result.headers.as_slice(), &["c_a", "c_b", "c_c", "c_d"]);

        assert_eq!(result.data.len(), 2);

        assert_eq!(result.data[0].as_slice(), &["1", "2", "3, wibble", "4"]);
        assert_eq!(result.data[1].as_slice(), &["5", "6", "7", "8"]);
    }

    #[test]
    fn test_empty_lines() {
        let input = "c_a, c_b, c_c, c_d\n\n1,2,\"3, wibble\",4,\n\n5, 6, \"7\", 8\n";
        let result = parse_csv_string(input);

        assert_eq!(result.headers.as_slice(), &["c_a", "c_b", "c_c", "c_d"]);

        assert_eq!(result.data.len(), 2);

        assert_eq!(result.data[0].as_slice(), &["1", "2", "3, wibble", "4"]);
        assert_eq!(result.data[1].as_slice(), &["5", "6", "7", "8"]);
    }

    #[test]
    fn test_weird_line() {
        let input = "Name, Start date, Start time, End date, End Time, Description, URL, Latitude, Longitude
Tech Talks Pesda, 2025-07-30, 18:30, 2025-07-30, 20:30:00, \"Ymunwch â ni yng Nghanolfan Cefnfaes rhwng 6.30 ac 8.30pm ar nos Fercher 30 Gorffennaf am amrywiaeth o sgyrsiau sy'n ymwneud â thechnoleg a rhwydweithio. Eisiau rhoi cyflwyniad? Oes gennych chi rywbeth i'w rannu? Cysylltwch â ni!

Join us at Canolfan Cefnfaes between 6.30 and 8.30pm on Wednesday the 30th July for a range of tech related talks and networking. Want to talk? Got something to share? Get in touch!\", https://www.eventbrite.co.uk/e/tech-talks-pesda-tickets-1446947301329, 53.181196, -4.064392";

        let result = parse_csv_string(input);
        assert_eq!(
            result.headers.as_slice(),
            &[
                "Name",
                "Start date",
                "Start time",
                "End date",
                "End Time",
                "Description",
                "URL",
                "Latitude",
                "Longitude"
            ]
        );

        assert_eq!(
            result.data[0].as_slice(),
            &[
                "Tech Talks Pesda",
                "2025-07-30",
                "18:30",
                "2025-07-30",
                "20:30:00",
                "Ymunwch Ã¢ ni yng Nghanolfan Cefnfaes rhwng 6.30 ac 8.30pm ar nos Fercher 30 Gorffennaf am amrywiaeth o sgyrsiau sy'n ymwneud Ã¢ thechnoleg a rhwydweithio. Eisiau rhoi cyflwyniad? Oes gennych chi rywbeth i'w rannu? Cysylltwch Ã¢ ni!\n\nJoin us at Canolfan Cefnfaes between 6.30 and 8.30pm on Wednesday the 30th July for a range of tech related talks and networking. Want to talk? Got something to share? Get in touch!",
                "https://www.eventbrite.co.uk/e/tech-talks-pesda-tickets-1446947301329",
                "53.181196",
                "-4.064392"
            ]
        )
    }
}
