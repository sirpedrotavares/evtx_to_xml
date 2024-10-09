# evtx_to_xml

A Rust-based tool for parsing and filtering Windows EVTX event logs. This tool processes both individual EVTX files and directories containing multiple EVTX files. It supports filtering logs by event IDs, date ranges, and specific target users.

## Features

- Parse individual `.evtx` files or directories of `.evtx` files.
- Filter logs by:
  - Specific event IDs (e.g., logon events, authentication events, etc.)
  - Date ranges (start date and end date)
  - Target usernames from a file
- Multithreading support for fast log processing using Rayon.

## Usage

### Installation

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

Options
--input_path: Path to the .evtx file or directory to process.
--output_file: Path to the output XML file.
--users_file: Path to the file containing a list of owned users (optional).
--start_date: Start date for filtering logs (format: YYYY-MM-DD) (optional).
--end_date: End date for filtering logs (format: YYYY-MM-DD) (optional).
--threads: Number of threads for multithreading (optional, default: system max).
