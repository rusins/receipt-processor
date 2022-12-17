use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::str;

use clap::Parser;
use receipt_processor::accounting;

use receipt_processor::receipt::Receipt;

#[derive(Parser)]
struct CliArguments {
    // Receipt file or folder in which to find receipt files
    path: PathBuf,
}

fn main() -> std::io::Result<()> {
    let args = CliArguments::parse();
    let metadata = fs::metadata(args.path.as_path())?;
    let files: Vec<PathBuf> = if metadata.is_file() {
        vec!(args.path)
    } else {
        let find_output = Command::new("find")
            .arg("-L") // follow links
            .arg(args.path.as_path())
            .args(&["-not", "-path", "*/[@.]*", "-type", "f"]) // ignore hidden files
            .output()
            .expect("failed to execute command to find files")
            .stdout;
        str::from_utf8(&find_output).unwrap().split("\n")
            .filter(|s| !s.is_empty())
            .map(|s| PathBuf::from(s))
            .collect()
    };

    let mut receipts = Vec::<Receipt>::new();
    for file in files {
        let file_name = file.file_name().unwrap().to_str().unwrap();
        if !file_name.ends_with(".check") {
            println!("WARN: Ignoring file {} because its file extension is not .check", file.as_path().to_str().unwrap());
        } else {
            match Receipt::parse(file) {
                Err(error) => println!("ERROR: Failed to parse file {}", error),
                Ok(receipt) => {
                    receipts.push(receipt)
                }
            }
        }
    }

    accounting::summary(receipts);

    Result::Ok(())
}


