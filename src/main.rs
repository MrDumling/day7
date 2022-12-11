use std::fs::File as InputFile;
use std::io::{BufRead, BufReader, Lines};

struct Directory {
    name: String,
    held_files: Vec<DataFile>,
    sub_directories: Vec<Directory>,
    path_directory: Option<String>,
}

impl Directory {
    fn get_pathed_directory(&mut self) -> Option<Self> {
        match &self.path_directory {
            Some(path) => {
                let index = self
                    .sub_directories
                    .iter()
                    .position(|x| &x.name == path)
                    .expect(&format!("Expected {path} to exist in instance")[..]);

                Some(self.sub_directories.remove(index))
            }
            None => None,
        }
    }

    fn push_path(&mut self, path_name: String) {
        if self.path_directory == None {
            self.path_directory = Some(path_name);
        } else {
            let mut pathed_directory = self.get_pathed_directory().unwrap();
            pathed_directory.push_path(path_name);

            let current_path = self.path_directory.clone();
            self.path_directory = None;
            self.push_directory(pathed_directory);
            self.path_directory = current_path;
        }
    }

    fn pop_path(&mut self) {
        let Some(mut sub_directory) = self.get_pathed_directory() else {
            panic!("Cannot go up from directory that has no path");
        };

        if sub_directory.path_directory == None {
            self.path_directory = None;
            self.sub_directories.push(sub_directory);
            return;
        }

        sub_directory.pop_path();
        self.sub_directories.push(sub_directory);
    }

    fn push_directory(&mut self, pushed_directory: Directory) {
        match self.get_pathed_directory() {
            Some(mut pathed_directory) => {
                pathed_directory.push_directory(pushed_directory);
                self.sub_directories.push(pathed_directory);
            }
            None => self.sub_directories.push(pushed_directory),
        };
    }

    fn push_file(&mut self, pushed_file: DataFile) {
        match self.get_pathed_directory() {
            Some(mut pathed_directory) => {
                pathed_directory.push_file(pushed_file);
                self.sub_directories.push(pathed_directory);
            }
            None => self.held_files.push(pushed_file),
        };
    }

    fn get_size(&self) -> u64 {
        (self
            .sub_directories
            .iter()
            .map(|x| x.get_size())
            .sum::<u64>())
            + (self.held_files.iter().map(|x| x.content_size).sum::<u64>())
    }
}

struct DataFile {
    content_size: u64,
}

fn get_input(path: &str) -> Lines<BufReader<InputFile>> {
    let file = InputFile::open(path).unwrap();
    let reader = BufReader::new(file);

    reader.lines()
}

enum CommandType {
    List,
    ChangeDirectory { directory_name: String },
}

impl CommandType {
    fn parse_instruction(input: &str) -> Self {
        if let Some(directory_name) = input.strip_prefix("cd ") {
            CommandType::ChangeDirectory {
                directory_name: String::from(directory_name),
            }
        } else {
            CommandType::List
        }
    }
}

enum ListedFile {
    GeneralFile { size: u64 },
    Directory { name: String },
}

impl ListedFile {
    fn parse_line(line: &str) -> Self {
        if let Some(directory_name) = line.strip_prefix("dir ") {
            ListedFile::Directory {
                name: directory_name.to_string(),
            }
        } else {
            let Some((size, _)) = line.split_once(' ') else {
                panic!("Invalid listed file: {line}");
            };

            ListedFile::GeneralFile {
                size: size.parse().unwrap(),
            }
        }
    }
}

enum InstructionLine {
    Command(CommandType),
    ListContent(ListedFile),
}

fn parse_input(lines: Lines<BufReader<InputFile>>) -> Vec<InstructionLine> {
    let mut result = Vec::new();

    for line in lines {
        let line = line.unwrap();
        result.push(if let Some(command_contents) = line.strip_prefix("$ ") {
            InstructionLine::Command(CommandType::parse_instruction(command_contents))
        } else {
            InstructionLine::ListContent(ListedFile::parse_line(&line))
        });
    }

    result
}

fn construct_file_system(instructions: Vec<InstructionLine>) -> Directory {
    let mut root = Directory {
        name: String::from("/"),
        held_files: Vec::new(),
        sub_directories: Vec::new(),
        path_directory: None,
    };

    for current_instruction in instructions {
        match current_instruction {
            InstructionLine::Command(CommandType::List) => {}
            InstructionLine::Command(CommandType::ChangeDirectory { directory_name }) => {
                if directory_name == "/" {
                    continue;
                } else if directory_name == ".." {
                    root.pop_path();
                } else {
                    root.push_path(directory_name);
                }
            }
            InstructionLine::ListContent(listed_file) => match listed_file {
                ListedFile::GeneralFile { size } => {
                    root.push_file(DataFile { content_size: size });
                }
                ListedFile::Directory { name } => {
                    root.push_directory(Directory {
                        name,
                        held_files: Vec::new(),
                        sub_directories: Vec::new(),
                        path_directory: None,
                    });
                }
            },
        }
    }

    root
}

fn puzzle_1() {
    let input = get_input("input.txt");
    let instructions = parse_input(input);

    let root = construct_file_system(instructions);
    let mut small_file_sum = 0u64;
    let mut queued_directories = vec![&root];

    while !queued_directories.is_empty() {
        let current_directory = queued_directories.pop().unwrap();
        queued_directories.extend(&current_directory.sub_directories);

        let current_directory_size = current_directory.get_size();
        if current_directory_size < 100_000 {
            small_file_sum += current_directory_size;
        }
    }

    println!("{small_file_sum}");
}


fn puzzle_2() {
    let input = get_input("input.txt");
    let instructions = parse_input(input);

    let root = construct_file_system(instructions);
    
    let total_size = root.get_size();
    let required_deleted = total_size - 40_000_000;
    let mut smallest_deleted = total_size;

    let mut queued_directories = vec![&root];

    while !queued_directories.is_empty() {
        let current_directory = queued_directories.pop().unwrap();
        let current_directory_size = current_directory.get_size();
        if current_directory_size >= required_deleted  {
            queued_directories.extend(&current_directory.sub_directories);
            smallest_deleted = std::cmp::min(current_directory_size, smallest_deleted);
        }
    }

    println!("{smallest_deleted}");
}

fn main() {
    puzzle_1();
    puzzle_2();
}
