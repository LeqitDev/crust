use std::{fs::{metadata, create_dir, File, read_dir}, borrow::Cow::{Owned, Borrowed, self}, process::Command};

use serde::{Serialize, Deserialize};

use rustyline::{Config, Editor, Result, error::ReadlineError, completion::Completer, Context, highlight::Highlighter, Helper, Validator, hint::Hinter};

use indoc::indoc;

#[macro_use]
extern crate diesel;

extern crate dotenv;

mod db;
use db::*;


const VERSION: &str = "0.0.2";
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

#[derive(Debug, Serialize, Deserialize)]
struct Path {
    projects: Vec<String>,
    prefix: String,
}

fn main() -> Result<()> {
    let mut create_tables = false;
    // create config folder if not yet existing
    if !metadata("config").is_ok() {
        create_dir("config").expect("Couldn't create config folder!");
        if !metadata("config/db.sqlite3").is_ok() {
            File::create("config/db.sqlite3").expect("Couldn't create projects file!");
            create_tables = true;
        }
    }
    
    let help_msg = indoc! {"
    \x1b[1;32mhelp\x1b[0m: print this little help text
    \x1b[1;32mexit\x1b[0m: exits the application
    \x1b[1;32madd-location [path] [prefix]\x1b[0m: adds a new location (parent folder) with projectfolder inside
                                  the prefix is used to diffirentiate between same projectnames in different locations
                                  it has to be unique
    \x1b[1;32mlist\x1b[0m: list all projects
    \x1b[1;32m[prefix].[projectname]\x1b[0m: opens the project. if the projectname is not known a plain rust project will be created in the folder of the prefix path"};

    let mut projects = get_all_projects(create_tables); // get projects from file

    let (mut complete_projects, mut commands) = initialize(&projects);

    let config = Config::builder().auto_add_history(true).build(); // set config with history
    let h = MyHelper {commands}; // create helper with commands
    
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
    ", format!("\x1b[32mv{}\x1b[0m", VERSION));

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
                        println!("{}", help_msg);
                    }
                    "exit" => {
                        println!("bye!");
                        break;
                    },
                    "list" => {
                        print_projects(&projects);
                    }
                    "add-location" => {
                        let (ok, projects_size) = add_path(args[0].to_string(), args[1].to_string()); // try adding path to resources
                        if ok { // path valid
                            println!("Successfully added '{}'{}!", args[0], format!(" and {} projects", projects_size));
                        } else {
                            println!("Couldn't add '{}'!", args[0]);
                        }
                    },
                    _ if complete_projects.contains(&cmd.to_string()) => {
                        let split: Vec<&str> = cmd.split(".").collect();
                        let prefix = split[0];
                        let project_name = split[1];
                        println!("Opening {} with Visual Studio Code...", &project_name);
    
                        let path = format!("{}\\{}", &projects.paths.iter().filter(|path| path.prefix == prefix.to_string()).next().unwrap().path, project_name); // retrieve path from prefix
                        Command::new("cmd").arg("/c").arg("code").arg(&path).spawn().expect("Couldn't run vscode command!"); // open with vscode
                    },
                    _ if cmd.contains(".") => {
                        let split = cmd.split(".").collect::<Vec<&str>>();
                        if let Some(path) = projects.paths.iter().filter(|p| p.prefix == split[0]).next() {
                            println!("Creating project {} at path {}", split[1], path.path);

                            let project_dir = format!("{}\\{}", path.path, split[1]);
                            Command::new("cmd").args(["/c", "cargo", "new", &project_dir]).status().expect("Couldn't create new rust project!"); // create new project

                            println!("Opening {} with Visual Studio Code...", &split[1]);
                            Command::new("cmd").arg("/c").arg("code").arg(&project_dir).spawn().expect("Couldn't run vscode command!"); // open with vscode

                            let _ = create_project(&split.get(1).unwrap(), &path.id);
                            projects = get_all_projects(false);
                            let commands = initialize(&projects).1;
                            let h = MyHelper {commands}; // create helper with commands
                            rl.set_helper(Some(h));
                        } else {
                            println!("Unknown command try 'help' for command list!");
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

fn print_projects(projects: &Index) {
    if projects.paths.len() == 0 {
        println!("No projects registered yet!\nTry 'add-location [path] [prefix]' to add a directory with projects!\n");
        return;
    }
    println!("Included projects:");
    for path_index in &projects.paths {
        println!("{}:", path_index.path);
        for project_name in &path_index.projects {
            println!("\t- {project_name}");
        }
    }
    println!("");
}

fn add_path(path: String, prefix: String) -> (bool, usize) {
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

    let path_id = create_path(&path, &prefix);

    for p_name in project_names {
        let _ = create_project(&p_name, &path_id);
    }

    // projects.add_path(path, Path {projects: project_names, prefix});
    (true, *projects_size)
}

fn is_valid_folder_path(path: &String) -> bool {
    if let Ok(metadata) = metadata(path) {
        metadata.is_dir()
    } else {
        false
    }
}

fn initialize(projects: &Index) -> (Vec<String>, Vec<String>) {
    let mut complete_projects = vec![];

    for path in &projects.paths {
        path.projects.iter().for_each(|p| complete_projects.push(format!("{}.{}", path.prefix, p)));
    }

    let mut commands = vec![
        "help".to_string(), 
        "exit".to_string(), 
        "add-location".to_string(), 
        "list".to_string(),
        ];

    commands.append(&mut complete_projects.clone());

    (complete_projects, commands)
}