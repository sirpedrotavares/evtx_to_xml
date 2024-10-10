# evtx_to_xml

A Rust-based tool for parsing and filtering Windows EVTX event logs. This tool processes both individual EVTX files and directories containing multiple EVTX files. It supports filtering logs by event IDs, date ranges, and specific target users.
The XML output can be uploaded into LogonTracer: https://github.com/JPCERTCC/LogonTracer

## Features

- Parse individual `.evtx` files or directories of `.evtx` files.
- Filter logs by:
  - Specific event IDs (e.g., logon events, authentication events, etc.)
  - Date ranges (start date and end date)
  - Target usernames from a file
- Multithreading support for fast log processing using Rayon.

### Installation and Usage

1. Clone the repository:

```bash
git clone https://github.com/yourusername/evtx_to_xml.git
cd evtx_to_xml
```

2. Build the project

```bash
cargo build --release
```

3. Running the Tool
```bash
./target/release/evtx_to_xml --input_path /path/to/evtx_or_directory --output_file output.xml [OPTIONS]
```

### Options

- `--input_path`: Path to the `.evtx` file or directory to process.
- `--output_file`: Path to the output XML file.
- `--users_file`: Path to the file containing a list of owned users (optional).
- `--start_date`: Start date for filtering logs (format: `YYYY-MM-DD`) (optional).
- `--end_date`: End date for filtering logs (format: `YYYY-MM-DD`) (optional).
- `--threads`: Number of threads for multithreading (optional, default: system max).

4. Execution output

```bash
./evtx_to_xml --input-path evtx_folder/ --output-file output.xml -u owned_users.txt --start-date 2024-03-18 --end-date 2024-08-26

Loading owned users from: owned_users.txt
Writing matched events to output file: output.xml
Processing EVTX file: security_evtx/1-Security.evtx
Processing EVTX file: security_evtx/2-Security.evtx
Processing EVTX file: security_evtx/3-Security.evtx
Processing EVTX file: security_evtx/4-Security.evtx
Processing EVTX file: security_evtx/5-Security.evtx
Processing EVTX file: security_evtx/6-Security.evtx
(..)
```

5. See how to import the XML into LogonTracer here:
```bash
https://gitbook.seguranca-informatica.pt/resources-1/windows-eventviewer-analysis-or-dfir#import-the-xml-into-logontracer
```
