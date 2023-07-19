use std::env;

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

    Index { paths: path_indexes }
}