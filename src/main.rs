#![windows_subsystem = "windows"]

use chrono::{DateTime, Duration, Local, Timelike};
use clap::Parser;
use directories::BaseDirs;
use rfd::FileDialog;
use ron::{from_str, to_string};
use serde::{Deserialize, Serialize};
use slint::{Color, PlatformError, SharedString, Timer, TimerMode};
use std::fs::create_dir_all;
use std::{
    env, env::current_dir, fs::read_to_string, fs::write, num::ParseIntError, ops::Deref,
    path::Path, path::PathBuf, sync::Arc, sync::RwLock, thread,
};

slint::include_modules!();

fn validate_string_inputs(s: &str, colon_allowed: bool) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut allowed_chars = vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
    if colon_allowed {
        allowed_chars.push(':');
    }

    for symbol in s.chars() {
        if !allowed_chars.contains(&symbol) {
            return false;
        }
    }
    true
}

fn get_new_color(valid: bool, background: slint::Color) -> slint::Brush {
    if valid {
        if background == Color::from_argb_u8(255, 250, 250, 250) {
            Color::from_argb_u8(230, 0, 0, 0).into()
        } else if background == Color::from_argb_u8(255, 28, 28, 28) {
            Color::from_argb_u8(255, 255, 255, 255).into()
        } else {
            Color::from_rgb_u8(0, 0, 0).into()
        }
    } else {
        Color::from_rgb_u8(200, 12, 12).into()
    }
}

fn format_time(secs: i64) -> String {
    let duration = Duration::seconds(secs);

    if duration.num_minutes() > 59 {
        format!(
            "{:02}:{:02}:{:02}",
            duration.num_hours(),
            duration.num_minutes() % 60,
            duration.num_seconds() % 60
        )
    } else {
        format!(
            "{:02}:{:02}",
            duration.num_minutes(),
            duration.num_seconds() % 60
        )
    }
}

fn get_seconds_until_time(target_time: &str) -> Result<u32, PlatformError> {
    let current: DateTime<Local> = Local::now();

    let mut parts: Vec<u32> = target_time
        .split(':')
        .map(|x| {
            x.parse::<u32>()
                .map_err(|e| PlatformError::Other(e.to_string()))
        })
        .collect::<Result<Vec<u32>, PlatformError>>()?; // Magically transform Iterator<Item=Result<_,_>> to Result<Vec<_>,_>

    if parts.len() == 1 {
        parts.push(0);
    }
    if parts.len() == 2 {
        parts.push(0);
    }

    let mut target: DateTime<Local> = current
        .with_hour(parts[0])
        .ok_or(PlatformError::Other("No hours".to_string()))?
        .with_minute(parts[1])
        .ok_or(PlatformError::Other("No minutes".to_string()))?
        .with_second(parts[2])
        .ok_or(PlatformError::Other("No seconds".to_string()))?;

    let secs = target.signed_duration_since(current).num_seconds();

    Ok(if secs < 0 {
        target += Duration::days(1);
        target.signed_duration_since(current).num_seconds() as u32
    } else {
        secs as u32
    })
}

fn get_seconds_from_mixed_format(input: &str) -> Result<Duration, ParseIntError> {
    let parts: Vec<&str> = input.split(':').collect();

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
        Ok(_) => {}
        Err(error) => {
            println!("{}", error)
        }
    };

    if verbose {
        println!("{}", line);
    }
}

fn count_me_down(
    seconds: u32,
    prefix: &str,
    ending: &str,
    step: usize,
    filepath: &str,
    verbose: bool,
) -> Result<(), PlatformError> {
    let current = Local::now();
    let end = current
        .checked_add_signed(Duration::seconds(seconds as i64))
        .ok_or(PlatformError::Other("Could not add signed".to_string()))?;

    let mut countdown_seconds: i64 = seconds.into();

    while Local::now().timestamp() < end.timestamp() {
        let line = format!("{} {}", prefix, format_time(countdown_seconds));

        write_to_file(&line, filepath, verbose);

        countdown_seconds -= step as i64;
        thread::sleep(std::time::Duration::from_secs(step as u64));
    }

    write_to_file(ending, filepath, verbose);
    Ok(())
}

#[derive(Deserialize, Serialize, Default)]
struct RustMeDownConfig {
    time_in: String,
    prefix: String,
    ending: String,
    step: usize,
    filepath: String,
}

impl RustMeDownConfig {
    fn from_serialized_config() -> Option<RustMeDownConfig> {
        if let Some(base_dirs) = BaseDirs::new() {
            let mut config_file = base_dirs.config_local_dir().to_path_buf();
            config_file.push("CountMeDown/countMeDown.ron");
            if config_file.exists() {
                let serialized_data = read_to_string(config_file);
                if let Ok(data) = serialized_data {
                    let config = from_str::<RustMeDownConfig>(data.as_str());
                    if let Ok(data) = config {
                        return Some(data);
                    }
                }
            }
        }
        None
    }

    fn serialize_to_disk(&self) {
        if let Some(base_dirs) = BaseDirs::new() {
            let mut config_file = base_dirs.config_local_dir().to_path_buf();
            config_file.push("CountMeDown/countMeDown.ron");
            let config_file_copy = config_file.clone();
            let _ = create_dir_all(config_file_copy.parent().unwrap()); // Safe unwrap, as a file always has a parent.

            let serialized = to_string(&self);
            if let Ok(config) = serialized {
                write(config_file, config).expect("Could not save config");
            }
        }
    }

    fn default() -> Self {
        RustMeDownConfig {
            time_in: "10:00".into(),
            prefix: "Start in:".into(),
            ending: "gleich".into(),
            step: 1usize,
            filepath: current_dir()
                .unwrap()
                .join("time.txt")
                .to_str()
                .unwrap()
                .into(),
        }
    }

    fn get_seconds(&self) -> Result<u32, ParseIntError> {
        let parts: Vec<&str> = self.time_in.split(':').collect();

        let mut factor: i64 = 1;
        let mut seconds: i64 = 0;

        for part in parts.into_iter().rev() {
            let s: i64 = part.parse()?;
            seconds += factor * s;
            factor *= 60;
        }

        Ok(seconds as u32)
    }
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

fn text_or_default(s: SharedString, default: &str) -> String {
    if s.as_str().to_owned().is_empty() {
        default.into()
    } else {
        s.as_str().to_owned()
    }
}

fn get_texts(ui: &CountMeDownGUI) -> (String, String, String) {
    let time_in = text_or_default(ui.get_time_text(), "10:00");
    let prefix = text_or_default(ui.get_prefix_text(), "");
    let ending = text_or_default(ui.get_ending_text(), "");
    (time_in, prefix, ending)
}

macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

fn main() -> Result<(), slint::PlatformError> {
    let prefix: String;
    let ending: String;
    let step: usize;
    let filepath: String;
    let verbose: bool;
    let seconds: u32;

    let args: Vec<String> = env::args().collect();
    let any_args = args.len() > 1;

    if any_args {
        // let seconds: u32 = 15;
        // let prefix = "start in: ";
        // let ending = "miau";
        // let step: usize = 2;
        // let filepath = "./time.txt";
        // let verbose: bool = true;
        let cli = Cli::parse();

        seconds = if cli.until {
            get_seconds_until_time(&cli.time_in)?
        } else {
            get_seconds_from_mixed_format(&cli.time_in)
                .map_err(|e| slint::PlatformError::Other(e.to_string()))?
                .num_seconds() as u32
        };

        prefix = cli.prefix.unwrap_or("".into());
        ending = cli.ending.unwrap_or("".into());
        step = cli.step;
        filepath = cli.file.unwrap_or("./time.txt".into());
        verbose = cli.verbose;

        let _ = count_me_down(seconds, &prefix, &ending, step, &filepath, verbose);
        Ok(())
    } else {
        let ui = CountMeDownGUI::new()?;

        let weak = ui.as_weak();

        let remaining_seconds_lock = Arc::new(RwLock::new(0u32));
        let remaining_lock_write = Arc::clone(&remaining_seconds_lock);
        let remaining_lock_timer = Arc::clone(&remaining_seconds_lock);

        let step_lock = Arc::new(RwLock::new(3usize));
        let step_lock_write = Arc::clone(&step_lock);

        let timer_run_lock = Arc::new(RwLock::new(false));
        let timer_run_lock_timer = Arc::clone(&timer_run_lock);
        let timer_run_lock_write = Arc::clone(&timer_run_lock);

        let timer = Timer::default();

        let run_config_lock = Arc::new(RwLock::new(RustMeDownConfig::default()));
        let run_config_write = Arc::clone(&run_config_lock);
        let run_config_timer = Arc::clone(&run_config_lock);

        ui.on_check_time_in(enclose! { (weak) move |val: SharedString| {
            let valid = validate_string_inputs(&val, true);
            let ui = weak.unwrap();

            ui.set_time_valid(valid);
            let new_color = get_new_color(valid, ui.get_bg());

            ui.set_time_in_label(new_color);
        }});

        ui.on_check_step_in(enclose! { (weak) move |val: SharedString| {
            let valid = validate_string_inputs(&val, false);
            let ui = weak.unwrap();

            ui.set_step_valid(valid);
            let new_color = get_new_color(valid, ui.get_bg());
            ui.set_step_label_color(new_color);
        }});

        ui.on_open_file_dialog(enclose! { (weak) move |path| {
            let ui = weak.unwrap();
            let startfile: PathBuf;
            let filename: &str;
            if path == "Pick" {
                startfile = current_dir().unwrap().join("time.txt");
                filename = "time.txt";
            } else {
                startfile = PathBuf::from(path.as_str());
                filename = startfile.file_name().unwrap().to_str().unwrap();
            }

            let savefile = FileDialog::new()
                .add_filter("text", &["txt"])
                .set_directory(startfile.parent().unwrap())
                .set_file_name(filename)
                .save_file();

            match savefile {
                Some(ref file) => {
                    ui.set_file_path(file.to_str().unwrap().into());
                    ui.set_file_name(file.file_name().unwrap().to_str().unwrap().into());
                }
                None => {
                    ui.set_file_path(startfile.to_str().unwrap().into());
                    ui.set_file_name(startfile.file_name().unwrap().to_str().unwrap().into());
                }
            }
        }});

        ui.on_run_clicked(enclose! { (weak) move |enabled| {
            let ui = weak.unwrap();

            if enabled {
                println!("Time: {}", ui.get_time_text());
                println!("Step: {}", ui.get_step_size());
                println!("File: {}", ui.get_file_path());
                println!("Prefix: {}", ui.get_prefix_text());
                println!("Ending: {}", ui.get_ending_text());

                let (time_in, prefix, ending) = get_texts(&ui);

                let filepath: String = ui.get_file_path().as_str().to_owned();

                let step: usize = ui.get_step_size().parse().unwrap_or(1);

                {
                    let mut write_config = run_config_write.write().unwrap();
                    *write_config = RustMeDownConfig {
                        time_in,
                        prefix,
                        ending,
                        step,
                        filepath,
                    };

                    let mut write_timer_run = timer_run_lock_write.write().unwrap();
                    *write_timer_run = !*write_timer_run.deref();

                    let mut write_step = step_lock_write.write().unwrap();
                    *write_step = write_config.step.to_owned();

                    let mut write_remaining = remaining_lock_write.write().unwrap();
                    *write_remaining = write_config.get_seconds().unwrap_or(600);
                }

                /*                let _ = count_me_down(
                    config.get_seconds().unwrap(),
                    config.prefix.as_str(),
                    config.ending.as_str(),
                    config.step,
                    config.filepath.as_str(),
                    verbose,
                );*/
            }
        }});

        ui.on_run_save(enclose! { (weak) move |enabled| {
            let ui = weak.unwrap();

            if enabled {
                let (time_in, prefix, ending) = get_texts(&ui);

                let filepath: String = ui.get_file_path().as_str().to_owned();

                let step: usize = ui.get_step_size().parse().unwrap_or(1);

                let config = RustMeDownConfig {
                    time_in,
                    prefix,
                    ending,
                    step,
                    filepath,
                };

                config.serialize_to_disk();

                ui.set_title_field("Cofig saved!".into());
            }
        }});

        ui.on_run_load(enclose! { (weak) move |enabled| {
            let ui = weak.unwrap();

            if enabled {
                let config = RustMeDownConfig::from_serialized_config();

                if let Some(config) = config {
                    let file_path = Path::new(&config.filepath);
                    ui.set_file_name(file_path.file_name().unwrap().to_str().unwrap().into());
                    ui.set_file_path(config.filepath.into());
                    ui.set_step_size(config.step.to_string().into());
                    ui.set_ending_text(config.ending.into());
                    ui.set_prefix_text(config.prefix.into());
                    ui.set_time_text(config.time_in.into());
                };
            }
        }});

        let mut steps: u32 = 0;

        timer.start(
            TimerMode::Repeated,
            std::time::Duration::from_secs(1),
            enclose! { (weak)
            move || {
                let ui = weak.unwrap();

                let remaining_guard = remaining_lock_timer.read().unwrap();
                let remaining_seconds = remaining_guard.deref();

                let read_timer_run = timer_run_lock_timer.read().unwrap();
                let mut finished: bool = false;
                if *read_timer_run.deref() && !finished {
                    let sep = if steps % 2 == 0 { ":" } else { " " };

                    let actual_remaining: i64 = (remaining_seconds - steps) as i64;
                    let line = format!(
                        "{} {}",
                        ui.get_prefix_text(),
                        format_time(actual_remaining).replace(':', sep)
                    );

                    ui.set_title_field(line.to_string().into());

                    let path = {
                        let config_guard = run_config_timer.read().unwrap();
                        config_guard.deref().filepath.to_string()
                    };

                    write_to_file(&line.to_string(), &path, true);

                    steps += 1;
                    if steps > *remaining_seconds {
                        finished = true;
                    }
                }
                if finished {
                    let (ending, path) = {
                        let config_guard = run_config_timer.read().unwrap();
                        let ending = config_guard.deref().ending.to_string();
                        let path = config_guard.deref().filepath.to_string();
                        (ending, path)
                    };

                    let title = {
                        if ending.is_empty() {
                            "CountMeDown".into()
                        } else {
                            ending.to_string()
                        }
                    };

                    ui.set_title_field(title.into());

                    write_to_file(&ending.to_string(), &path, true);
                }
        }});

        ui.run()
    }
}
