use std::{
    collections::HashMap,
    fs::{self, DirEntry},
    io::{BufRead, BufReader, Seek},
    time::SystemTime,
};

use imgui::{TreeNodeFlags, Ui};

pub struct CategorizedSpoilers {
    pub category: CategoryType,
    pub item_map: Vec<(Item, Location)>,
}

impl CategorizedSpoilers {
    pub fn new(category: CategoryType) -> Self {
        Self {
            category,
            item_map: vec![],
        }
    }

    pub fn render(&self, ui: &Ui, show_tt: bool) {
        let label = format!("{:?}", self.category);
        if ui.collapsing_header(label, TreeNodeFlags::DEFAULT_OPEN) {
            for (item, location) in &self.item_map {
                let frmt_item = format!("{:<width$}", item.name, width = 32);
                ui.text(frmt_item);
                ui.same_line();
                ui.text(&location.short);
                if show_tt && ui.is_item_hovered() {
                    let tt_token = ui.begin_tooltip();
                    let wrap_token = ui.push_text_wrap_pos_with_pos(300.);
                    ui.text(&location.long);
                    wrap_token.end();
                    tt_token.end();
                }
            }
        }
    }

    pub fn find_long_descriptions(&mut self, file: &mut fs::File) -> std::io::Result<()> {
        let item_names: Vec<String> = self.item_map.iter().map(|p| p.0.name.clone()).collect();
        file.rewind()?;

        let reader = BufReader::new(file);
        let mut map: HashMap<String, &mut (Item, Location)> = HashMap::new();
        for pair in self.item_map.iter_mut() {
            map.insert(pair.0.name.clone(), pair);
        }

        // Read through the file and check for the pattern "item name + 'in'"
        // Don't love this, might be a better way
        for line in reader.lines() {
            match line {
                Ok(text) => {
                    for name in &item_names {
                        let concat = format!("{} in", name);
                        if text.contains(concat.as_str()) {
                            match map.get_mut(name) {
                                Some(pair) => pair.1.long = text.to_owned(),
                                None => {}
                            }
                        }
                    }
                }
                Err(_) => continue,
            }
        }
        Ok(())
    }
}

pub struct Location {
    pub short: String,
    pub long: String,
}

pub struct Item {
    pub name: String,
}

#[derive(Debug)]
pub enum CategoryType {
    Key,
    Quest,
    Upgrade,
    Healing,
}

pub struct Spoilers {
    pub source_file: String,
    pub key_items: CategorizedSpoilers,
    pub quest_items: CategorizedSpoilers,
    pub upgrade_items: CategorizedSpoilers,
    pub healing_items: CategorizedSpoilers,
}

impl Spoilers {
    pub fn new() -> Self {
        Self {
            source_file: String::new(),
            key_items: CategorizedSpoilers::new(CategoryType::Key),
            quest_items: CategorizedSpoilers::new(CategoryType::Quest),
            upgrade_items: CategorizedSpoilers::new(CategoryType::Upgrade),
            healing_items: CategorizedSpoilers::new(CategoryType::Healing),
        }
    }

    pub fn read_recent(&mut self) -> std::io::Result<()> {
        let mut exe_path = std::env::current_exe().unwrap();
        exe_path.push("..\\randomizer\\spoiler_logs");
        
        let parent_dir = exe_path.as_path();

        let read_dir = fs::read_dir(parent_dir)?;

        let mut file_infos = vec![];
        for f in read_dir {
            file_infos.push(f?);
        }

        file_infos.sort_by(|a, b| creation_date(a).cmp(&creation_date(b)));
        file_infos.reverse();

        if file_infos.first().is_none() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No files in spoioler directory!",
            ));
        }

        let recent = file_infos.first().unwrap();
        self.read_file(recent.path().to_str().unwrap())
    }

    pub fn read_file(&mut self, path: &str) -> std::io::Result<()> {
        self.source_file = path.to_owned();
        let mut file = fs::File::open(&self.source_file)?;
        Spoilers::populate_hints_from_pattern(
            &mut file,
            &mut self.key_items,
            "-- Hints for key items:",
        )?;
        Spoilers::populate_hints_from_pattern(
            &mut file,
            &mut self.quest_items,
            "-- Hints for quest items:",
        )?;
        Spoilers::populate_hints_from_pattern(
            &mut file,
            &mut self.upgrade_items,
            "-- Hints for upgrade items:",
        )?;
        Spoilers::populate_hints_from_pattern(
            &mut file,
            &mut self.healing_items,
            "-- Hints for healing items:",
        )?;

        self.key_items.find_long_descriptions(&mut file)?;
        self.quest_items.find_long_descriptions(&mut file)?;
        self.upgrade_items.find_long_descriptions(&mut file)?;
        self.healing_items.find_long_descriptions(&mut file)?;

        Ok(())
    }

    // will clear item map. always do this before geting long hints
    fn populate_hints_from_pattern(
        file: &mut fs::File,
        spoiler_chunk: &mut CategorizedSpoilers,
        pattern: &str,
    ) -> std::io::Result<()> {
        let lines = text_between(file, pattern, "");
        if lines.is_err() {
            return Err(lines.err().unwrap());
        }

        spoiler_chunk.item_map.clear();

        let lines_ok = lines.unwrap();
        for line in lines_ok {
            let mut item: (Item, Location) = (
                Item {
                    name: String::new(),
                },
                Location {
                    short: String::new(),
                    long: String::new(),
                },
            );
            for (i, split) in line.split(":").enumerate() {
                if i == 0 {
                    item.0 = Item {
                        name: split.to_owned(),
                    };
                }
                if i == 1 {
                    item.1 = Location {
                        short: split.to_owned(),
                        long: String::new(),
                    }
                }
            }
            spoiler_chunk.item_map.push(item);
        }

        Ok(())
    }
}

fn text_between(
    file: &mut fs::File,
    start: &str,
    end: &str,
) -> Result<Vec<String>, std::io::Error> {
    file.rewind()?;
    let reader = BufReader::new(file);
    let mut found_hints = false;
    let mut hint_lines = vec![];

    for line in reader.lines() {
        let mut string = line?;
        string = string.trim().to_owned();

        if found_hints {
            if end == string {
                return Ok(hint_lines);
            } else {
                hint_lines.push(string.to_owned());
            }
        }

        if string.contains(start) {
            found_hints = true;
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("Read to end of file without finding match ({})", &start),
    ))
}

fn creation_date(entry: &DirEntry) -> SystemTime {
    return entry.metadata().unwrap().created().unwrap();
}
