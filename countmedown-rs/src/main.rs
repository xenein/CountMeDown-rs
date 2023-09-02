use std::num::ParseIntError;
use std::fs::write;
use std::thread;
use chrono::{Duration, DateTime, Local, Timelike};
use clap::Parser;

fn format_time(secs: i64) -> String {

    let duration = Duration::seconds(secs);

    if duration.num_minutes() > 59 {
        format!("{:02}:{:02}:{:02}", duration.num_hours(), duration.num_minutes() % 60, duration.num_seconds() % 60)
    } else {
        format!("{:02}:{:02}", duration.num_minutes(), duration.num_seconds() % 60)
    }
}

fn get_seconds_until_time(target_time: &str) -> u32 {
    let current: DateTime<Local> = Local::now();

    let mut parts: Vec<u32> =  target_time.split(":").map(|x| x.parse::<u32>().unwrap()).collect();

    if parts.len() == 1 {
        parts.push(0);
    }
    if parts.len() == 2 {
        parts.push(0);
    }

    let mut target: DateTime<Local> = current.with_hour(parts[0]).unwrap().with_minute(parts[1]).unwrap().with_second(parts[2]).unwrap();
    
    let secs = target.signed_duration_since(current).num_seconds();

    if secs < 0 {
        target = target + Duration::days(1);
        target.signed_duration_since(current).num_seconds() as u32
    } else {
        secs as u32
    }
}


fn get_seconds_from_mixed_format(input: &str) -> Result<Duration, ParseIntError> {
    let parts: Vec<&str> = input.split(":").collect();

    let mut factor: i64 = 1;
    let mut seconds: i64 = 0;

    for part in parts.into_iter().rev() {
        let s: i64 = part.parse()?;
        seconds += factor * s;
        factor *= 60;
    }

    Ok(Duration::seconds(seconds))
}

fn write_to_file(line: &str, filepath: &str, verbose: bool) {
    match write(filepath, line) {
        Ok(_) => {},
        Err(error) => {println!("{}", error)} 
    };
    if verbose {
        println!("{}", line);
    }
}

fn count_me_down( seconds: u32, prefix: &str, ending: &str, step: usize, filepath: &str, verbose: bool) {
    let current = Local::now();
    let end = current.checked_add_signed(Duration::seconds(seconds as i64)).unwrap();

    let mut countdown_seconds: i64 = seconds.into();
    while Local::now().timestamp() < end.timestamp() {
        let line = format!("{} {}", prefix, format_time(countdown_seconds));
        write_to_file(&line, filepath, verbose);
        countdown_seconds = countdown_seconds - step as i64;
        thread::sleep(std::time::Duration::from_secs(step as u64));
    }

    write_to_file(ending, filepath, verbose)

}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(default_value_t = false, short = 'v', long)]
    verbose: bool,
    #[arg(short = 'f', long)]
    file: Option<String>,
    #[arg(short = 's', long, default_value_t = 1)]
    step: usize,
    #[arg(short = 'p', long)]
    prefix: Option<String>,
    #[arg(short = 'e', long)]
    ending: Option<String>,
    time_in: String,
    #[arg(short = 'u', long, default_value_t = false)]
    until: bool,
}

fn main() {
    // let seconds: u32 = 15;
    // let prefix = "start in: ";
    // let ending = "miau"; 
    // let step: usize = 2;
    // let filepath = "./time.txt";
    // let verbose: bool = true;
    let cli = Cli::parse();
    
    let seconds;
    if cli.until {
        seconds = get_seconds_until_time(&cli.time_in);
    } else {
        seconds = get_seconds_from_mixed_format(&cli.time_in).unwrap().num_seconds() as u32;
    }

    let prefix = cli.prefix.unwrap_or("".into());
    let ending = cli.ending.unwrap_or("".into());
    let step = cli.step;
    let filepath = cli.file.unwrap_or("./time.txt".into());
    let verbose = cli.verbose;
    


    count_me_down(seconds, &prefix, &ending, step, &filepath, verbose)  
}
