# Friday
The idea of this program is a rather simplified console analog of jenkins for running various jobs with configuration in yml.
Each job  can be run on a schedule, have several steps with commands, the results of execution are recorded in the work log, 
and artifacts can be collected in the same directory as log.

## Get started
  Install with:
  - CARGO: `cargo install --git https://github.com/alhazred/friday.git`

### Configuration
The main friday config file is `.config/config.yml`. You can define `homedir`, as place for the job logs and artifacts, and the common friday log file name.
Jobs configs are located in the `.config/jobs/` directory (see `sample.yml`). Each job should have own config file. It's possible to use multiple steps for one job.
Schedule time uses GMT time, you can get your machine GMT time using `date -u` command.


### How to compile
Use the `Cargo` tool to get dependencies automatically downloaded.
Steps:
```
cargo build --release
```
Then take a look at the `target/release` folder.

