use serde::{Serialize, Deserialize};

use super::schema::{paths, projects};

#[derive(Serialize, Deserialize, Queryable, Selectable, Identifiable, Debug)]
#[diesel(table_name = paths)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Path {
    pub id: String,
    pub path: String,
    pub prefix: String,
}

#[derive(Insertable)]
#[table_name = "paths"]
pub struct NewPath<'a> {
    pub id: &'a str,
    pub path: &'a str,
    pub prefix: &'a str,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Identifiable, Debug)]
#[diesel(table_name = projects)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path_id: String,
}

#[derive(Insertable)]
#[table_name = "projects"]
pub struct NewProject<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub path_id: &'a str,
}