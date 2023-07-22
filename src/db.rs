use std::env;
use std::fs::read_dir;

use diesel::{SqliteConnection, Connection, sql_query};
use diesel::prelude::*;
use dotenv::dotenv;
use uuid::Uuid;

use self::models::{NewPath, Project, NewProject};
use self::schema::projects::{dsl::*};
use self::{schema::paths::{dsl::*}, models::Path};

pub mod models;
pub mod schema;

pub struct Index {
    pub paths: Vec<PathIndex>,
}

pub struct PathIndex {
    pub id: String,
    pub path: String,
    pub prefix: String,
    pub projects: Vec<String>, // TODO: Add a whole object with description or other metadata
}

fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set!");
    SqliteConnection::establish(&database_url).unwrap_or_else(|_| panic!("Error connecting to {database_url}!"))
}

pub fn create_path(path_path: &str, path_prefix: &str) -> String {
    let mut connection = establish_connection();

    let uuid = Uuid::new_v4().hyphenated().to_string();

    let new_path = NewPath {id: &uuid, path: path_path, prefix: path_prefix};

    diesel::insert_into(paths)
        .values(&new_path)
        .execute(&mut connection)
        .expect("Error saving new path!");
    
    uuid
}

pub fn create_project(project_name: &str, project_path_id: &str) -> String {
    let mut connection = establish_connection();

    let uuid = Uuid::new_v4().hyphenated().to_string();

    let new_project = NewProject {id: &uuid, name: project_name, path_id: project_path_id };

    diesel::insert_into(projects)
        .values(&new_project)
        .execute(&mut connection)
        .expect("Error saving new porject");

    uuid
}

pub fn get_project(project_name: &str, project_path_id: &str) -> Option<Project> {
    let mut connection = establish_connection();

    if let Ok(found_projects) = projects.filter(name.eq(project_name)).filter(path_id.eq(project_path_id)).limit(1).load::<Project>(&mut connection) {
        if found_projects.len() == 1 {
            return Some((found_projects as Vec<Project>).get(0).unwrap().clone())
        }
    }

    None
}

pub fn remove_project(project_name: &str, project_path_id: &str) {
    let mut connection = establish_connection();

    diesel::delete(&get_project(project_name, project_path_id).unwrap()).execute(&mut connection).expect("Error deleting project!");
}

pub fn get_all_projects(create_tables: bool) -> Index {
    let mut connection = establish_connection();
    if create_tables {
        sql_query(include_str!("create_paths.sql")).execute(&mut connection).expect("Couldn't create paths table!");
        sql_query(include_str!("create_projects.sql")).execute(&mut connection).expect("Couldn't create projects table!");
    }
    let mut path_indexes: Vec<PathIndex> = vec![];

    let ps = paths.load::<Path>(&mut connection).expect("Error loading paths!");
    for p in ps {
        let uuid = p.id.clone();
        let project_list: Vec<Project> = projects.filter(path_id.eq(p.id)).load::<Project>(&mut connection).expect("Error loading projects!");
        path_indexes.push(PathIndex { id: uuid, path: p.path, prefix: p.prefix, projects: project_list.iter().map(|p| &p.name).cloned().collect() });
    }

    let mut index = Index { paths: path_indexes };

    update_path_projects(&mut index);

    index
}

fn update_path_projects(index: &mut Index) {
    for p in &mut index.paths {
        let mut project_lookup: Vec<String> = vec![];
        if let Ok(entries) = read_dir(&p.path) { // read all directorys in path

            for entry in entries {
                if let Ok(entry) = entry {
                    if entry.path().is_dir() {
                        let project_name = entry.file_name().to_str().unwrap().to_string();
                        if let Ok(project_entries) = read_dir(entry.path()) { // read all files in directory
    
                            for project_entry in project_entries {
                                if let Ok(project_entry) = project_entry {
                                    if project_entry.file_type().unwrap().is_file() {
                                        if project_entry.file_name().to_str().unwrap() == "Cargo.toml" { // directory contains Cargo.toml -> rust project
                                            if !project_indexed(&project_name, &p.id) {
                                                create_project(&project_name, &p.id);
                                                p.projects.push(project_name.clone());
                                            }
                                            project_lookup.push(project_name.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        let mut remove_indexes: Vec<usize> = vec![];
        for (index, path_project) in p.projects.iter().enumerate() {
            if !project_lookup.contains(&path_project) {
                remove_project(&path_project, &p.id);
                remove_indexes.push(index);
            }
        }
        remove_indexes.iter().for_each(|index| { p.projects.remove(*index); });
    }
}

fn project_indexed(project_name: &str, project_path_id: &str) -> bool {
    if get_project(project_name, project_path_id).is_some() {
        true
    } else {
        false
    }
}