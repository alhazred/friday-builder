use std::time::Duration;
use std::fs;
use log::{info, error, debug};
use simplelog::{WriteLogger};
use std::path::{Path, PathBuf};
use job_scheduler::{Job, JobScheduler};
use colored::Colorize;
use std::process;
use std::collections::HashMap;
use crate::job::execute_job;

mod config;
mod job;

fn setup_logger(config: &config::Config) {
    if let Some(parent) = Path::new(&config.log_file).parent() {
        fs::create_dir_all(parent).expect("Unable to create log directory");
    }

    let log_config = simplelog::ConfigBuilder::new()
        .set_time_format_custom(simplelog::format_description!(
            "[month repr:short] [day] [hour]:[minute]:[second]"
        ))
        .build();

    WriteLogger::init(
        config.get_level_filter(),
        log_config,
        fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&config.log_file)
            .expect("Unable to open log file")
    ).expect("Unable to initialize logger");
}

fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs() % 60;
    let minutes = (duration.as_secs() / 60) % 60;
    let hours = (duration.as_secs() / 60) / 60;
    format!("{} hours {} minutes {} seconds", hours, minutes, seconds)
}

#[tokio::main]
async fn main() {
    info!("{}", "Reading config...".green());
    let config_path = PathBuf::from(config::CONFIG_PATH);
    let friday_cfg = config_path.join("config.yml");
    if !friday_cfg.exists() {
        error!("Config file {} does not exist, exiting", friday_cfg.display());
        process::exit(1);
    }

    let config = match config::Config::new(friday_cfg.clone()) {
        Ok(cfg) => cfg,
        Err(err) => {
            error!("Error reading config: {}", err);
            process::exit(1);
        }
    };

    debug!("{}: {}", "Homedir".green(), config.homedir.display().to_string());
    debug!("{}: {}", "Log file".green(), config.log_file);
    debug!("{}: {}", "Log level".green(), config.log_level);

    setup_logger(&config);

    let homedir = config.homedir;
 
    let mut scheduler = JobScheduler::new();

    let jobs_dir = config_path.join("jobs");

    if !jobs_dir.exists() {
        error!("Directory {} does not exist, exiting", jobs_dir.display());
        process::exit(1);
    }

    let mut jobs_counter = false;
    for entry in fs::read_dir(jobs_dir.clone()).unwrap() {
        let entry = entry.unwrap();
        let os_str = entry.file_name();
        let name = os_str.to_str().unwrap();

        let cfg = jobs_dir.join(name);

        if let Ok(jconfig) = config::Job::new(cfg) {
            info!("friday started");

            if !jconfig.job.is_empty() {
                let job = jconfig.job.as_str().replace(" ","_");
                let s = jconfig.cron.as_str();

                info!("Scheduling job: {} Schedule: {}", job, s);
                jobs_counter = true;

                let jobdir = homedir.join(job);
                if !jobdir.exists() {
                    fs::create_dir(&jobdir).expect("failed to create job subdirectory.");
                }
            
                let mut steps = HashMap::new();

                for element in jconfig.steps.clone() {
                    let step_name: String = element["name"].as_str().unwrap().replace(" ", "_");
                    let command: String = element["command"].as_str().unwrap().to_string();
                    info!("Step to run: {} , command: {}", step_name, command);
                    steps.insert(command.into(), step_name.into());
                }

                let mut artifacts: HashMap<String, String> = HashMap::new();

                for element in jconfig.artifacts.clone() {
                    let ws: String = element["workspace"].as_str().unwrap().to_string();
                    let files: String = element["files"].as_str().unwrap().to_string();
                    info!("Artifacts ws: {} , files: {}", ws, files);
                    artifacts.insert(ws.into(), files.into());
                }

                scheduler.add(Job::new(s.parse().unwrap(), move || {
                    execute_job(&steps, &artifacts, &jobdir);
                }));
            }
        }
    }

    if jobs_counter {
        info!("Scheduler activated...");
        loop {
            scheduler.tick();
            let duration = scheduler.time_till_next_job();
            debug!("Sleep {} for the next job ...", format_duration(duration));
            std::thread::sleep(duration);
        }
    } else {
        info!("No jobs found, exiting");
    }
}
