use evtx::{EvtxParser, ParserSettings};
use evtx::err::Result;
use rayon::prelude::*;
use clap::Parser;
use std::fs::{File, OpenOptions, read_dir};
use std::io::{Write, BufWriter, BufRead, BufReader};
use std::path::Path;
use std::sync::Mutex;
use serde_xml_rs::from_str; // For XML deserialization
use serde::Deserialize;
use chrono::{NaiveDateTime, DateTime, Utc, TimeZone};

/// Command-line arguments structure
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the .evtx file or directory
    #[arg(short, long)]
    input_path: String,

    /// Path to the output XML file
    #[arg(short, long)]
    output_file: String,

    /// Path to the file with the list of owned users (optional)
    #[arg(short, long)]
    users_file: Option<String>,

    /// Start date for filtering logs (format: YYYY-MM-DD) (optional)
    #[arg(short = 's', long)]
    start_date: Option<String>,

    /// End date for filtering logs (format: YYYY-MM-DD) (optional)
    #[arg(short = 'e', long)]
    end_date: Option<String>,

    /// Optional number of threads (default is system maximum)
    #[arg(short, long, default_value_t = num_cpus::get())]
    threads: usize,
}

// The Event IDs we want to include
const EVENT_IDS: &[u16] = &[4624, 4625, 4768, 4769, 4776, 4672];

fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Inicialize o pool de threads global de Rayon **uma vez** no início do programa
    rayon::ThreadPoolBuilder::new()
        .num_threads(args.threads)
        .build_global()
        .expect("Failed to initialize global thread pool");

    // Load owned users from file (if provided)
    let owned_users = if let Some(users_file) = args.users_file {
        println!("Loading owned users from: {}", users_file);
        load_owned_users(&users_file)
    } else {
        println!("No owned users file provided, processing all events.");
        Vec::new()  // Empty list means all users
    };

    // Parse the start and end dates (if provided)
    let start_date = args.start_date.as_ref().map(|d| parse_date(d));
    let end_date = args.end_date.as_ref().map(|d| parse_date(d));

    println!("Writing matched events to output file: {}", args.output_file);
    
    // Open the output file with a buffered writer for efficiency
    let output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&args.output_file)?;
    let output_writer = Mutex::new(BufWriter::new(output_file));

    // Check if the input path is a directory or a file
    let input_path = Path::new(&args.input_path);
    if input_path.is_dir() {
        // If input is a directory, process all .evtx files in the directory
        for entry in read_dir(input_path).expect("Failed to read directory") {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "evtx") {
                process_evtx_file(path.to_str().unwrap(), &owned_users, start_date, end_date, &output_writer);
            }
        }
    } else if input_path.is_file() {
        // If input is a single file, process the file
        process_evtx_file(&args.input_path, &owned_users, start_date, end_date, &output_writer);
    } else {
        println!("Invalid input path. Please provide a valid file or directory.");
    }

    println!("Processing complete. Check the output file for matched events.");
    Ok(())
}

/// Process a single .evtx file and append the results to the output
fn process_evtx_file(evtx_file: &str, owned_users: &[String], start_date: Option<DateTime<Utc>>, end_date: Option<DateTime<Utc>>, output_writer: &Mutex<BufWriter<File>>) {
    println!("Processing EVTX file: {}", evtx_file);
    
    // Open the EVTX file
    let mut parser = EvtxParser::from_path(evtx_file).expect("Failed to open EVTX file");

    // Process records in parallel using rayon
    parser.records()
        .par_bridge()  // Parallelize the iterator
        .for_each(|record| {
            match record {
                Ok(record) => {
                    // Get the XML data from the record
                    let xml_output = record.data.clone();  // Clone the entire XML

                    // Only process the record if its EventID matches one of the ones we care about
                    if let Some(event_id) = get_event_id_from_xml(&xml_output) {
                        if EVENT_IDS.contains(&event_id) {
                            // Get the event timestamp
                            if let Some(event_time) = get_time_created_from_xml(&xml_output) {
                                // Check if the event falls within the specified date range
                                if in_date_range(&event_time, start_date, end_date) {
                                    // Check if we need to filter by TargetUserName
                                    if owned_users.is_empty() {
                                        // No user file provided, write the full XML event
                                        let mut writer = output_writer.lock().unwrap();
                                        writeln!(writer, "{}", xml_output).unwrap();
                                    } else {
                                        // User file is provided, filter by TargetUserName
                                        if let Some(target_user_name) = get_target_user_name_from_xml(&xml_output) {
                                            if owned_users.contains(&target_user_name) {
                                                // Write the matched XML event
                                                let mut writer = output_writer.lock().unwrap();
                                                writeln!(writer, "{}", xml_output).unwrap();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Error processing record: {}", err);
                }
            }
        });
}

/// Function to get the EventID from the XML string
fn get_event_id_from_xml(xml_str: &str) -> Option<u16> {
    // Deserialize the XML and extract the EventID
    if let Ok(event) = from_str::<Event>(xml_str) {
        Some(event.system.event_id)
    } else {
        None
    }
}

/// Function to get the TargetUserName from the XML string
fn get_target_user_name_from_xml(xml_str: &str) -> Option<String> {
    if let Ok(event) = from_str::<Event>(xml_str) {
        for data in event.event_data.data {
            if data.name == "TargetUserName" {
                return Some(data.value);
            }
        }
    }
    None
}

/// Function to get the TimeCreated from the XML string
fn get_time_created_from_xml(xml_str: &str) -> Option<DateTime<Utc>> {
    if let Ok(event) = from_str::<Event>(xml_str) {
        if let Some(system_time) = event.system.time_created {
            return parse_time_created(&system_time.system_time);
        }
    }
    None
}

/// Check if the event timestamp falls within the provided date range
fn in_date_range(event_time: &DateTime<Utc>, start: Option<DateTime<Utc>>, end: Option<DateTime<Utc>>) -> bool {
    if let Some(start) = start {
        if *event_time < start {
            return false;
        }
    }
    if let Some(end) = end {
        if *event_time > end {
            return false;
        }
    }
    true
}

/// Parse a date in the format YYYY-MM-DD
fn parse_date(date_str: &str) -> DateTime<Utc> {
    Utc.datetime_from_str(&format!("{} 00:00:00", date_str), "%Y-%m-%d %H:%M:%S")
        .expect("Invalid date format. Use YYYY-MM-DD.")
}

/// Parse the TimeCreated field from the XML into a DateTime<Utc>
fn parse_time_created(time_str: &str) -> Option<DateTime<Utc>> {
    // Parse the "SystemTime" field in the format "2024-08-18 13:45:55.479781 UTC"
    NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S%.f UTC")
        .ok()
        .map(|naive_dt| DateTime::<Utc>::from_utc(naive_dt, Utc))
}

/// Structure to represent the Event XML
#[derive(Deserialize, Debug)]
struct Event {
    #[serde(rename = "System")]
    system: System,

    #[serde(rename = "EventData")]
    event_data: EventData,
}

/// Structure to represent the System element in the Event XML
#[derive(Deserialize, Debug)]
struct System {
    #[serde(rename = "EventID")]
    event_id: u16,

    #[serde(rename = "TimeCreated", default)]
    time_created: Option<TimeCreated>,
}

/// Structure to represent the TimeCreated element in the System XML
#[derive(Deserialize, Debug)]
struct TimeCreated {
    #[serde(rename = "SystemTime")]
    system_time: String,
}

/// Structure to represent the EventData element in the Event XML
#[derive(Deserialize, Debug)]
struct EventData {
    #[serde(rename = "Data", default)]
    data: Vec<Data>,
}

/// Structure to represent individual Data elements in EventData
#[derive(Deserialize, Debug)]
struct Data {
    #[serde(rename = "Name", default)]
    name: String,

    #[serde(rename = "$value", default)]
    value: String,
}

/// Load the owned users from a file
fn load_owned_users(file_path: &str) -> Vec<String> {
    let file = File::open(file_path).expect("Failed to open the users file.");
    let reader = BufReader::new(file);

    reader.lines()
        .map(|line| line.expect("Could not read line"))
        .collect()
}

