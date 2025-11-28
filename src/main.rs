use std::collections::HashMap;

use clap::{Parser, command};
use ics::{
    Event, ICalendar,
    properties::{Description, DtEnd, DtStart, Summary},
};

use crate::csv::CSVData;

pub mod csv;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(index = 1, required = true, help = "Path to input csv file to read")]
    input_csv: String,

    #[arg(
        index = 2,
        required = true,
        help = "Path to output ics file to write to, will be overwritten by this if it exists."
    )]
    output_ics: String,

    #[arg(
        short,
        long,
        help = "Title to use for the calender",
        default_value = "makers_calendar_uk"
    )]
    title: String,
}

fn date_format(date: &str, time: &str) -> String {
    // 2025-07-19
    // 13:00:00

    // 20250719T130000

    let date_chars: Vec<char> = date.chars().collect();
    let time_chars: Vec<char> = time.chars().collect();

    let seconds = if time_chars.len() > 6 {
        extract(&time_chars, 6, 8)
    } else {
        "00".to_owned()
    };

    format!(
        "{}{}{}T{}{}{}",
        extract(&date_chars, 0, 4),
        extract(&date_chars, 5, 7),
        extract(&date_chars, 8, 10),
        extract(&time_chars, 0, 2),
        extract(&time_chars, 3, 5),
        seconds
    )
}

fn extract(source: &Vec<char>, start: usize, end: usize) -> String {
    if source.len() < start || source.len() < end {
        panic!(
            "Date format incorrect expected something like '2025-07-19' got {:?}, tried to get characters {} to {} but length is {}",
            source,
            start,
            end,
            source.len()
        );
    }
    return source[start..end].iter().collect::<String>();
}

fn build_header_map(csv_data: &CSVData) -> HashMap<&str, Option<usize>> {
    // Name, Start date, Start time, End date, End Time, Description, URL, Latitude, Longitude

    let mut result = HashMap::new();
    result.insert(
        "name",
        csv_data
            .headers
            .iter()
            .position(|h| h.to_lowercase() == "name"),
    );
    result.insert(
        "description",
        csv_data
            .headers
            .iter()
            .position(|h| h.to_lowercase() == "description"),
    );
    result.insert(
        "start date",
        csv_data
            .headers
            .iter()
            .position(|h| h.to_lowercase() == "start date"),
    );
    result.insert(
        "end date",
        csv_data
            .headers
            .iter()
            .position(|h| h.to_lowercase() == "end date"),
    );
    result.insert(
        "start time",
        csv_data
            .headers
            .iter()
            .position(|h| h.to_lowercase() == "start time"),
    );
    result.insert(
        "end time",
        csv_data
            .headers
            .iter()
            .position(|h| h.to_lowercase() == "end time"),
    );

    result
}

fn get_field(
    name: &str,
    row: &Vec<String>,
    heading_mapping: &HashMap<&str, Option<usize>>,
) -> Option<String> {
    heading_mapping
        .get(name)
        .and_then(|k| k.map(|i| row[i].clone()))
}

fn main() {
    let args = Args::parse();

    let csv_data = csv::parse_csv_file(&args.input_csv).expect("Could not parse csv file");

    let heading_mapping = build_header_map(&csv_data);

    let mut calendar = ICalendar::new("2.0", &args.title);

    for row in &csv_data.data {
        // generate event uid from event name
        let name = get_field("name", row, &heading_mapping);
        let start_date = get_field("start date", row, &heading_mapping);
        let start_time = get_field("start time", row, &heading_mapping);

        if name.is_none() || start_date.is_none() || start_time.is_none() {
            println!(
                "Row Missing required field Name: '{}' Start_date: '{}' start_time: '{}'",
                name.or(Some("".to_string())).unwrap(),
                start_date.or(Some("".to_string())).unwrap(),
                start_time.or(Some("".to_string())).unwrap()
            );
            continue;
        }

        let name = name.unwrap();
        let start_date = start_date.unwrap();
        let start_time = start_time.unwrap();

        println!("processing {} {} {}", &name, &start_date, &start_time);

        let start_date_time = date_format(&start_date, &start_time);

        let mut event = Event::new(name.clone(), start_date_time.clone());
        event.push(DtStart::new(start_date_time.clone()));

        if let Some(end_date) = get_field("end date", row, &heading_mapping)
            && let Some(end_time) = get_field("end time", row, &heading_mapping)
        {
            let end_date_time = date_format(&end_date, &end_time);
            event.push(DtEnd::new(end_date_time));
        }
        event.push(Summary::new(name.clone()));

        if let Some(description) = get_field("description", row, &heading_mapping) {
            event.push(Description::new(description.clone()));
        }

        calendar.add_event(event);
    }

    calendar
        .save_file(&args.output_ics)
        .expect("Could not write ics file");
}
