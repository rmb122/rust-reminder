use std::fs::create_dir_all;
use std::path;

use chrono::NaiveDateTime;
use diesel::Insertable;
use diesel::prelude::*;
use diesel::Queryable;

use super::schema::todo;

#[derive(Queryable)]
pub struct Todo<'a> {
    pub id: i32,
    pub content: &'a String,
    pub expire_time: Option<NaiveDateTime>,
}

#[derive(Insertable)]
#[table_name = "todo"]
pub struct NewTodo<'a> {
    pub content: &'a String,
    pub expire_time: Option<NaiveDateTime>,
}


pub fn db_new_todo(conn: &SqliteConnection, todo: &Todo) {
    let todo_entity = NewTodo {
        content: todo.content,
        expire_time: todo.expire_time,
    };
    diesel::insert_into(todo::table).values(&todo_entity).execute(conn).expect("Error saving new todo");
}

diesel_migrations::embed_migrations!("migrations/");
pub fn establish_connection(database_path: Option<String>) -> SqliteConnection {
    let mut real_database_path = String::from("~/.config/rust-reminder/todo.db"); // default value

    match database_path {
        Some(database_path) => {
            real_database_path = database_path;
        }
        None => {}
    }

    if real_database_path.starts_with("~") {
        real_database_path = std::env::var("HOME").expect("$HOME not fount") + &real_database_path[1..];
    }

    let real_database_path = path::Path::new(&real_database_path);
    create_dir_all(real_database_path.parent().unwrap()).expect(&format!("Error mkdir {}", real_database_path.to_str().unwrap()));
    let conn = SqliteConnection::establish(real_database_path.to_str().unwrap()).expect(&format!("Error connecting to {}", real_database_path.to_str().unwrap()));

    embedded_migrations::run(&conn);
    return conn;
}