pub const VERSION: &str = "0.1";
pub const CONFIG_PATH: &str = "./config";
pub const DEFAULT_HOMEDIR: &str = "/var/lib/friday";

use yaml_rust::yaml::{Yaml, YamlLoader, Array};
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use simplelog::LevelFilter;
use colored::Colorize;

pub struct Config {
    pub version: &'static str,
    pub homedir: PathBuf,
    pub log_file: String,
    pub log_level: String
}

impl Config {

    pub fn new(config_path: PathBuf) -> Result<Self, String> {
        println!("{}: {}", "Loaded friday config from".green(), config_path.display());
        let yaml = read_config(&config_path)?;

        let homedir = yaml[0]["homedir"]
            .as_str()
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                println!("{} {}", "Homedir not set, using default: ".yellow(), DEFAULT_HOMEDIR);
                PathBuf::from(DEFAULT_HOMEDIR)
        });

        let log_file = yaml[0]["log"]["file"]
            .as_str()
            .map(String::from)
            .ok_or("log->file not found in config.yml.".to_string())?;

        let log_level = yaml[0]["log"]["level"]
             .as_str()
            .map(String::from)
            .unwrap_or_else(|| {
                println!("{}", "log->level not found in config.yml, using 'info'.".yellow());
                String::from("info")
            });

        Ok(Config {
            version: VERSION,
            homedir,
            log_file,
            log_level
        })
    }


    pub fn get_level_filter(&self) -> LevelFilter {
        let mut log = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(self.log_file.clone())
            .expect("(get_level_filter) Unable to open events log file.");

        match self.log_level.as_str() {
            "debug" | "Debug" | "DEBUG" | "D" | "d" => LevelFilter::Debug,
            "info" | "Info" | "INFO" | "I" | "i" => LevelFilter::Info,
            "error" | "Error" | "ERROR" | "E" | "e" => LevelFilter::Error,
            "warning" | "Warning" | "WARNING" | "W" | "w" | "warn" | "Warn" | "WARN" => LevelFilter::Warn,
            _ => {
                let msg = String::from("invalid log level from 'config.yml', using Info level.").red();
                println!("{}", msg);
                writeln!(log, "{}", msg).expect("cannot write in log file.");
                LevelFilter::Info
            }
        }
    }
}


pub struct Job {
    pub job: String,
    pub steps: Array,
    pub artifacts: Array,
    pub cron: String
}

impl Job {

    pub fn new(config_path: PathBuf) -> Result<Self, String> {
        println!("{}: {}", "Loaded job from".green(), config_path.display());
        let yaml = read_config(&config_path)?;

        let job = yaml[0]["job"]["name"]
            .as_str()
            .map(String::from)
            .unwrap_or_else(|| String::from("Not_used"));

        let cron = yaml[0]["schedule"]["time"]
            .as_str()
            .map(String::from)
            .unwrap_or_else(|| String::from("Not_used"));

        let steps = yaml[0]["steps"]
            .as_vec()
            .map_or(Vec::new(), |value| value.to_vec());

        let artifacts = yaml[0]["artifacts"]
            .as_vec()
            .map_or(Vec::new(), |value| value.to_vec());


        Ok(Job {
            job,
            steps,
            artifacts,
            cron
        })
    }

}

pub fn read_config(path: &PathBuf) -> Result<Vec<Yaml>, String> {
    let mut file = File::open(path).map_err(|e| format!("Unable to open file '{}': {}", path.display(), e))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Unable to read file '{}': {}", path.display(), e))?;
    YamlLoader::load_from_str(&contents).map_err(|e| format!("Error parsing YAML: {}", e))
}

