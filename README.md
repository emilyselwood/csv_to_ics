# csv to ics converter

This is designed to be used  with [calendar page](https://github.com/emilyselwood/calendar_page) to build an ics file from the csv of events.

```bash
cargo run -- path/to/csv/file.csv path/to/desired/ics/file.ics

```

Order of the columns shouldn't matter. However it will need timestamps split into date and time columns with the format being yyyy-mm-dd for dates and hh:mm:ss for times.

The following column names will be recognized in the input csv. Any other columns will be ignored.

Column | description | required | format
-------------------------------
Name   | Name of the event | True | 
Start Date | Date that the event starts on | True | yyyy-mm-dd
Start Time | Time that the event starts | True | hh:mm:ss
Description | longer description of the event | False |
End Date | Date that the event ends on | False | yyyy-mm-dd
End Time | Time that the event ends | False | hh:mm:ss
