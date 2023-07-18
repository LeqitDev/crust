use std::{fs::{metadata, create_dir, read_to_string, File, read_dir, write}, io::Write, borrow::Cow::{Owned, Borrowed, self}, collections::HashMap};

use serde::{Serialize, Deserialize};

use rustyline::{Config, Editor, Result, error::ReadlineError, completion::Completer, Context, highlight::Highlighter, Helper, Validator, hint::Hinter};

// Helper Struct
#[derive(Helper, Validator)]
struct MyHelper {
    commands: Vec<String>, // Commands used by completer and hinter
}

impl Completer for MyHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<String>)> {
        // Autovervollständigung der Befehle basierend auf dem eingegebenen Text
        let completions: Vec<_> = self.commands
            .iter()
            .filter(|command| command.starts_with(line)) // starts with typed characters
            .map(|command| command[line.len()..].to_string()) // return commands - typed characters
            .collect(); // return all possible commands

        Ok((pos, completions))
    }
}

impl Hinter for MyHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        if line.is_empty() || pos < line.len() {
            return None;
        }

        let hint = self.commands
            .iter()
            .filter(|command| command.starts_with(line)) // starts with typed characters
            .map(|command| command[line.len()..].to_string()) // return commands - typed characters
            .next(); // return only one possible

        hint
    }
}

impl Highlighter for MyHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Owned(format!("\x1b[1;32m{prompt}\x1b[0m")) // highlight pompt ('> ')
        } else {
            Borrowed(prompt)
        }
    }

    // highlight hint ansi colors
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned(format!("\x1b[2m{hint}\x1b[0m"))
    }
}

// json object holding all paths + projects
// TODO: add default workspace others need prefix
// TODO: add description or other metadata
#[derive(Debug, Serialize, Deserialize)]
struct Projects {
    #[serde(flatten)]
    paths: HashMap<String, Vec<String>>,
}

impl Projects {
    fn fetch_projects() -> Projects {
        // try to read config/projects.json or else create it
        let file = read_to_string("config/projects.json").unwrap_or_else(|_| {
            let mut f = File::create("config/projects.json").expect("Couldn't create projects file!");
            f.write_all(b"{}").expect("Couldn't write to packages.json!");
            read_to_string("config/projects.json").expect("Couldn't read created projects file!")
        });
    
        // path json to json object if empty create new empty json object
        match serde_json::from_str::<Projects>(&file) {
            Ok(projects) => projects,
            Err(_) => Projects { paths: HashMap::default() },
        }
    }

    fn add_path(&mut self, path: String, projects: Vec<String>) {
        self.paths.insert(path, projects); // add to json object

        // save to config/projects.json
        let json_string = serde_json::to_string_pretty(&self).expect("Couldn't convert object to pretty string!");
        let _ = write("config/projects.json", json_string);
    }
}

fn main() -> Result<()> {
    // create config folder if not yet existing
    if !metadata("config").is_ok() {
        create_dir("config").expect("Couldn't create config folder!");
    }

    let config = Config::builder().auto_add_history(true).build(); // set config with history
    // TODO: create builder ::new(Vec<&str>) -> MyHelper
    let h = MyHelper {
        commands: vec![
            "help".to_string(), 
            "exit".to_string(), 
            "exit now".to_string(),
            "exit not-now".to_string(),
            "add-location".to_string(), 
            "list".to_string()
            ]
        }; // create helper with commands
    
    // load sqlhistory
    let history = if false {
        // memory
        rustyline::sqlite_history::SQLiteHistory::with_config(config)?
    } else {
        // file
        rustyline::sqlite_history::SQLiteHistory::open(config, "config/history.sqlite3")?
    };

    // create editor and set helper
    let mut rl: Editor<MyHelper, _> = Editor::with_history(config, history)?;
    rl.set_helper(Some(h));
    
    let mut projects = Projects::fetch_projects(); // get projects from file

    // welcome msg
    println!("Welcome to...");
    println!(r"
   ██████╗ ██████╗  ██╗   ██╗ ███████╗ ████████╗
  ██╔════╝ ██╔══██╗ ██║   ██║ ██╔════╝ ╚══██╔══╝
  ██║      ██████╔╝ ██║   ██║ ███████╗    ██║
  ██║      ██╔══██╗ ██║   ██║ ╚════██║    ██║
  ╚██████╗ ██║  ██║ ╚██████╔╝ ███████║    ██║
   ╚═════╝ ╚═╝  ╚═╝  ╚═════╝  ╚══════╝    ╚═╝   
  {}
    ", format!("\x1b[32mv0.0.1\x1b[0m"));

    print_projects(&projects);

    loop {
        let line = rl.readline("> ");
        match line {
            Ok(s) => {
                let split: Vec<&str> = s.split(" ").collect();
                let cmd = split[0];
                let args: Vec<&str> = split[1..].to_vec();

                match cmd {
                    "help" => {

                    }
                    "exit" => {
                        println!("bye!");
                        break;
                    },
                    "add-location" => {
                        let (ok, projects_size) = add_path(&mut projects, args[0].to_string()); // try adding path to resources
                        if ok { // path valid
                            println!("Successfully added '{}'{}!", args[0], format!(" and {} projects", projects_size));
                        } else {
                            println!("Couldn't add '{}'!", args[0]);
                        }
                    }
                    _ => println!("Unknown command try 'help' for command list!"),
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("bye!");
                break;
            }
            Err(err) => {
                println!("Error: {err:?}");
                break;
            }
        }
    }
    Ok(())
}

fn print_projects(projects: &Projects) {
    if projects.paths.len() == 0 {
        println!("No projects registered yet!\nTry 'add-location [path]' to add a directory with projects!\n");
        return;
    }
    println!("Included projects:");
    for (path, project_names) in &projects.paths {
        println!("{path}:");
        for project_name in project_names {
            println!("\t- {project_name}");
        }
    }
    println!("");
}

fn add_path(projects: &mut Projects, path: String) -> (bool, usize) {
    if !is_valid_folder_path(&path) {
        return (false, 0);
    }
    let mut project_names = vec![];
    if let Ok(entries) = read_dir(&path) { // read all directorys in path

        for entry in entries {
            if let Ok(entry) = entry {
                if entry.path().is_dir() {
                    let project_name = entry.file_name().to_str().unwrap().to_string();
                    if let Ok(project_entries) = read_dir(entry.path()) { // read all files in directory

                        for project_entry in project_entries {
                            if let Ok(project_entry) = project_entry {
                                if project_entry.file_type().unwrap().is_file() {
                                    if project_entry.file_name().to_str().unwrap() == "Cargo.toml" { // directory contains Cargo.toml -> rust project
                                        project_names.push(project_name.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let projects_size = &project_names.len();
    projects.add_path(path, project_names);
    (true, *projects_size)
}

fn is_valid_folder_path(path: &String) -> bool {
    if let Ok(metadata) = metadata(path) {
        metadata.is_dir()
    } else {
        false
    }
}