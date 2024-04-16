use hudhook::hooks::dx11::ImguiDx11Hooks;
use hudhook::ImguiRenderLoop;
use imgui::*;
use std::fs::DirEntry;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::{Instant, SystemTime};

struct Nightjar {
    _start_time: Instant,
    hints: Hints,
    custom_log_dir: String,
}

impl Nightjar {
    fn new() -> Self {
        Self {
            _start_time: Instant::now(),
            hints: get_hints(
                "C:/Program Files (x86)/Steam/steamapps/common/Sekiro/randomizer/spoiler_logs",
            ),
            custom_log_dir:
                "C:/Program Files (x86)/Steam/steamapps/common/Sekiro/randomizer/spoiler_logs"
                    .to_string(),
        }
    }

    fn update_from_log_dir(&mut self) {
        self.hints = get_hints(&self.custom_log_dir);
    }
}

struct Hints {
    values: Option<Vec<(String, String)>>,
    errors: Vec<String>,
}

fn get_hints(dir: &str) -> Hints {
    let mut hint_data = Hints {
        values: None,
        errors: vec![],
    };

    let path = Path::new(dir);
    let files_read = std::fs::read_dir(path);
    if files_read.is_err() {
        hint_data.errors.push("Failed to find directory".to_owned());
        return hint_data;
    }

    let files = files_read.unwrap();

    println!("Looking for spoiler logs in {:?}", path.to_str());
    let mut file_infos = vec![];

    for f in files {
        match f {
            Ok(file) => file_infos.push(file),
            Err(e) => {
                eprintln!("{}", e);
                hint_data.errors.push(e.to_string());
                return hint_data;
            }
        }
    }

    file_infos.sort_by(|a, b| creation_date(a).cmp(&creation_date(b)));
    file_infos.reverse(); // Want the most recent, not least
    match file_infos.first() {
        Some(recent) => {
            hints_from_file(&mut hint_data, recent, "-- Hints for key items:");
        }
        None => {
            eprintln!("No recent spoiler log found.");
            hint_data
                .errors
                .push("No recent spoiler log found.".to_owned());
        }
    }

    return hint_data;
}

fn hints_from_file(data: &mut Hints, entry: &DirEntry, pattern: &str) {
    let mut hints = vec![];
    match std::fs::File::open(entry.path()) {
        Ok(file) => {
            let reader = BufReader::new(file);
            let mut found_hints = false;

            let mut line_count = -1;
            for line in reader.lines() {
                line_count += 1;
                match line {
                    Ok(line_text) => {
                        if found_hints {
                            // Check for whitespace, indicating the end of hints section
                            if line_text.trim().is_empty() {
                                data.values = Some(hints);
                                return;
                            }

                            // Else, try and split the line
                            let splits: Vec<&str> = line_text.split(':').collect();
                            if splits.len() < 2 {
                                data.errors.push(format!(
                                    "Unexpected hint format encountered at line {}",
                                    line_count
                                ));
                                continue;
                            }
                            hints.push((splits[0].to_owned(), splits[1].to_owned()));
                        } else if line_text.contains(pattern) {
                            found_hints = true;
                        }
                    }
                    Err(e) => {
                        data.errors.push(e.to_string());
                        continue;
                    }
                }
            }

            if found_hints {
                data.errors.push(format!(
                    "Found the start, but never the end? Read {} lines",
                    line_count
                ));
            }

            data.errors
                .push(format!("Failed to encounter pattern in {:?}", entry.path()));
            return;
        }
        Err(_) => {
            data.errors
                .push(format!("Failed to open file {:?}", entry.path()));
            return;
        }
    }
}

fn creation_date(entry: &DirEntry) -> SystemTime {
    return entry.metadata().unwrap().created().unwrap();
}

impl ImguiRenderLoop for Nightjar {
    fn render(&mut self, ui: &mut Ui) {
        let position: [f32; 2] = [0., 0.];
        ui.window("Nightjar Monocular")
            .size([600., 200.], Condition::Always)
            .position(position, Condition::Appearing)
            .build(|| {
                if ui.button("refresh") {
                    self.update_from_log_dir();
                }
                ui.same_line();
                ui.input_text("log directory", &mut self.custom_log_dir)
                    .build();

                match &self.hints.values {
                    Some(valid_hints) => {
                        for hint in valid_hints {
                            ui.text(format!(
                                "{} {}",
                                format!("{:<width$}", hint.0, width = 32),
                                hint.1
                            ));
                        }
                    }
                    None => {
                        ui.text("Could not find spoiler logs!");
                    }
                }

                for e in &self.hints.errors {
                    ui.text(e);
                }
            });
    }
}

hudhook::hudhook!(ImguiDx11Hooks, Nightjar::new());
