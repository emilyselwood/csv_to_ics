use std::env;

use ics::{
    Event, ICalendar,
    properties::{Description, DtEnd, DtStart, Location, Summary},
};

pub mod csv;

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
    return source[start..end].iter().collect::<String>();
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // the first arg is the executable file name.
    if args.len() != 3 {
        println!("Requires two positional arguments");
        println!("csv_to_ics <path/to/csv/file.csv> <path/to/ics/file.ics>");
        return;
    }

    let csv_path = args.get(1).unwrap();
    let ics_path = args.get(2).unwrap();

    let csv_data = csv::parse_csv_file(&csv_path).expect("could not parse csv file");

    let mut calendar = ICalendar::new("2.0", "makers_calendar_uk");

    for row in csv_data.data {
        // generate event uid from event name
        println!("processing {} {} {}", row[0], row[1], row[2]);

        let mut event = Event::new(row[0].clone(), date_format(&row[1], &row[2]));
        event.push(DtStart::new(date_format(&row[1], &row[2])));
        event.push(DtEnd::new(date_format(&row[3], &row[4])));
        event.push(Summary::new(row[0].clone()));
        event.push(Description::new(row[5].clone()));

        calendar.add_event(event);
    }

    calendar
        .save_file(ics_path)
        .expect("Could not write ics file");
}
