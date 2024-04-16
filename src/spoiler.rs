use std::{
    collections::HashMap,
    fs::{self, DirEntry},
    hash::Hash,
    io::{BufRead, BufReader},
    path::Path,
    time::SystemTime,
};

const DEFAULT_PATH: &str =
    "C:/Program Files (x86)/Steam/steamapps/common/Sekiro/randomizer/spoiler_logs";

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

    pub fn find_long_descriptions(&mut self, file: &fs::File) {
        let item_names: Vec<String> = self.item_map.iter().map(|p| p.0.name.clone()).collect();

        let reader = BufReader::new(file);
        let mut map: HashMap<String, &mut (Item, Location)> = HashMap::new();
        for pair in self.item_map.iter_mut() {
            map.insert(pair.0.name.clone(), pair);
        }

        // Read through the file and check for the pattern "item name + 'in'"
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
    }
}

pub struct Location {
    pub short: String,
    pub long: String,
}

pub struct Item {
    pub name: String,
}

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
        let parent_dir = Path::new(DEFAULT_PATH);
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
        let file = fs::File::open(&self.source_file)?;
        Spoilers::populate_hints_from_pattern(
            &file,
            &mut self.key_items,
            "-- Hints for key items:",
        )?;
        Spoilers::populate_hints_from_pattern(
            &file,
            &mut self.quest_items,
            "-- Hints for quest items:",
        )?;
        Spoilers::populate_hints_from_pattern(
            &file,
            &mut self.upgrade_items,
            "-- Hints for upgrade items:",
        )?;
        Spoilers::populate_hints_from_pattern(
            &file,
            &mut self.healing_items,
            "-- Hints for healing items:",
        )?;

        self.key_items.find_long_descriptions(&file);
        self.quest_items.find_long_descriptions(&file);
        self.upgrade_items.find_long_descriptions(&file);
        self.healing_items.find_long_descriptions(&file);

        Ok(())
    }

    // will clear item map. always do this before geting long hints
    fn populate_hints_from_pattern(
        file: &fs::File,
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
            }
            spoiler_chunk.item_map.push(item);
        }

        Ok(())
    }
}

fn text_between(file: &fs::File, start: &str, end: &str) -> Result<Vec<String>, std::io::Error> {
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
        "Read to end of file without finding match",
    ))
}

fn creation_date(entry: &DirEntry) -> SystemTime {
    return entry.metadata().unwrap().created().unwrap();
}
