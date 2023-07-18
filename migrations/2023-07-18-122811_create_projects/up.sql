-- Your SQL goes here
CREATE TABLE projects (
    id VARCHAR NOT NULL PRIMARY KEY UNIQUE,
    name TEXT NOT NULL,
    path_id VARCHAR NOT NULL
);