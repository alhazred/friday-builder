use std::fs;
use std::path::Path;
use std::io::BufReader;
use std::collections::HashMap;
use duct::cmd;
use std::io::prelude::*;
use regex::Regex;
use walkdir::WalkDir;
use log::{info, error, debug};

fn glob_to_regex(pattern: &str) -> String {
    let mut regex_pattern = String::new();

    for c in pattern.chars() {
        match c {
            '*' => regex_pattern.push_str(".*"),
            '.' | '(' | ')' | '+' | '|' | '[' | ']' | '^' | '$' | '?' | '{' | '}' | '\\' => {
                regex_pattern.push('\\');
                regex_pattern.push(c);
             }
             _ => regex_pattern.push(c),
         }
    }

    format!("^{}$", regex_pattern)
}


fn find_and_copy_artifacts(source_dir: &Path, target_dir: &Path, glob_pattern: &str) {
    let regex_pattern = glob_to_regex(glob_pattern);
    let regex = Regex::new(&regex_pattern).expect("Invalid regex pattern");
    for entry in WalkDir::new(source_dir) {
        let entry = entry.expect("Error reading directory entry");
        let path = entry.path();
        if path.is_file() && regex.is_match(path.file_name().unwrap().to_str().unwrap()) {
            let dest_path = target_dir.join(path.file_name().unwrap());
            match fs::copy(&path, &dest_path) {
                Ok(_) => {
                    debug!("Copied artifact: {} to {}", path.display(), dest_path.display());
                }
                Err(err) => {
                    error!("Error copying {}: {}", path.display(), err);
                    continue;
                }
            }
        }
    }
}


pub fn execute_job(steps: &HashMap<String, String>, artifacts: &HashMap<String, String>, jobdir: &Path) {
    let datetime = chrono::offset::Utc::now();
    let tm = datetime.format("%Y-%m-%d.%H:%M").to_string();
    let logdir = jobdir.join(tm);
    if !logdir.exists() {
          fs::create_dir(&logdir).expect("failed to create log subdirectory.");
    }

    for (_step, (command, name)) in steps.iter().enumerate() {
        info!("Running: {} step, command: {}", name, command);

        let targs: Vec<&str> = command.split_whitespace().collect();
        let out = cmd(targs[0], &targs[1..]);

        let reader = match out.stderr_to_stdout().reader() {
            Ok(reader) => reader,
            Err(err) => {
                error!("Error creating reader: {}", err);
                return;
            }
        };

        let stdout = logdir.join(name).with_extension("output");
        let mut lines = BufReader::new(reader).lines();
 
        let mut sd = match std::fs::File::create(&stdout) {
            Ok(file) => file,
            Err(err) => {
                error!("Error creating log file: {}", err);
                return; 
            }
        };

        writeln!(sd, "##################### STEP: {} #####################\n", name).expect("Cannot write to log file");
        while let Some(line) = lines.next() {
            if let Ok(line) = line {
                writeln!(sd, "{}", line).expect("Cannot write to log file");
            } else {
                error!("Error reading line from reader");
            }
        }
    }
    for (_af, (ws, files)) in artifacts.iter().enumerate() {
        info!("Job Artifacts ws: {}, files: {}", ws, files);
        let globs = files.split(",");
        for glob in globs {
            find_and_copy_artifacts(Path::new(ws), &logdir, glob.trim_start().trim_end());
        }
    }
}
