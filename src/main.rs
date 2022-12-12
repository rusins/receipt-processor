use std::fs;
use std::str;
use std::fs::File;
use std::io::BufReader;
use std::process::Command;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use clap::Parser;

#[derive(Parser)]
struct CliArguments {
    // Receipt file or folder in which to find receipt files
    path: std::path::PathBuf,
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
            .args(&["-name", "[^.]*", "-type", "f"]) // ignore hidden files
            .output()
            .expect("failed to execute command to find files")
            .stdout;
        str::from_utf8(&find_output).unwrap().split("\n")
            .filter(|s| !s.is_empty())
            .map(|s| PathBuf::from(s))
            .collect()
    };

    for file in files {
        println!("File name {}", file.file_name().unwrap().to_str().unwrap());
    }

    Result::Ok(())
}


