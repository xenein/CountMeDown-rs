use chrono::{DateTime, Duration, Local, Timelike};
use clap::Parser;
use rfd::FileDialog;
use slint::{Color, PlatformError, SharedString};
use std::{env, env::current_dir, fs::write, num::ParseIntError, path::PathBuf, thread};

slint::include_modules!();

fn validate_string_inputs(s: &str, colon_allowed: bool) -> bool {
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

        let ui_handle = ui.as_weak();
        let ui_handle2 = ui.as_weak();
        let ui_handle3 = ui.as_weak();
        let ui_handle_run = ui.as_weak();

        ui.on_check_time_in(move |val: SharedString| {
            let valid = validate_string_inputs(&val, true);
            let ui = ui_handle.unwrap();

            ui.set_time_valid(valid);
            let new_color = get_new_color(valid, ui.get_bg());

            ui.set_time_in_label(new_color);
        });

        ui.on_check_step_in(move |val: SharedString| {
            let valid = validate_string_inputs(&val, false);
            let ui = ui_handle3.unwrap();

            ui.set_step_valid(valid);
            let new_color = get_new_color(valid, ui.get_bg());
            ui.set_step_label_color(new_color);
        });

        ui.on_open_file_dialog(move |path| {
            let ui = ui_handle2.unwrap();
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
        });

        ui.on_run_clicked(move |enabled| {
            let ui = ui_handle_run.unwrap();
            if enabled {
                println!("Time: {}", ui.get_time_text());
                println!("Step: {}", ui.get_step_size());
                println!("File: {}", ui.get_file_path());
                println!("Prefix: {}", ui.get_prefix_text());
                println!("Ending: {}", ui.get_ending_text());

                let prefix: String = if ui.get_prefix_text().as_str().to_owned().is_empty() {
                    "".into()
                } else {
                    ui.get_prefix_text().as_str().to_owned()
                };

                let ending: String = if ui.get_ending_text().as_str().to_owned().is_empty() {
                    "".into()
                } else {
                    ui.get_ending_text().as_str().to_owned()
                };

                let filepath: String = ui.get_file_path().as_str().to_owned();

                let step: usize = ui.get_step_size().parse().unwrap_or(1);
                let seconds = get_seconds_from_mixed_format(ui.get_time_text().as_str())
                    .unwrap_or(Duration::seconds(600))
                    .num_seconds() as u32;
                let verbose = true;

                let _ = count_me_down(seconds, &prefix, &ending, step, &filepath, verbose);
            }
        });

        ui.run()
    }
}
