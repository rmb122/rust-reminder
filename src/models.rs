use std::borrow::Borrow;
use std::fs::create_dir_all;
use std::path;

use chrono::{Date, Local, NaiveDateTime};
use diesel::prelude::*;
use super::schema::todo;

#[derive(Queryable)]
pub struct Todo {
    pub id: i32,
    pub content: String,
    pub expire_time: Option<NaiveDateTime>,
}

impl Clone for Todo {
    fn clone(&self) -> Self {
        Todo{
            id: self.id.clone(),
            content: self.content.clone(),
            expire_time: self.expire_time.clone()
        }
    }
}

#[derive(Insertable)]
#[table_name = "todo"]
pub struct NewTodo {
    pub content: String,
    pub expire_time: Option<NaiveDateTime>,
}


pub fn db_new_todo(conn: &SqliteConnection, t: &NewTodo) {
    diesel::insert_into(todo::table).values(t).execute(conn).expect("Error saving new todo");
}

pub fn db_find_todo(conn: &SqliteConnection, date: Option<Date<Local>>) -> Vec<Todo> {
    match date {
        Some(date) => {
            let utc_date_time_start = date.and_hms(0, 0, 0).naive_utc();
            let utc_date_time_end = date.and_hms(23, 59, 59).naive_utc();

            todo::dsl::todo.filter(todo::dsl::expire_time.between(utc_date_time_start, utc_date_time_end)).order_by(todo::dsl::expire_time).load::<Todo>(conn).expect("Query error")
        }
        None => {
            todo::dsl::todo.filter(todo::dsl::expire_time.is_null()).order_by(todo::dsl::id).load::<Todo>(conn).expect("Query error")
        }
    }
}

pub fn db_del_todo(conn: &SqliteConnection, todo_id: &Vec<i32>) {
    if todo_id.len() <= 0 {
        return
    }
    diesel::delete(todo::table.filter(todo::id.eq_any(todo_id))).execute(conn).expect("Delete error");
}

pub fn db_update_todo(conn: &SqliteConnection, todo: &Todo) {
    diesel::update(
        todo::table.filter(todo::dsl::id.eq(todo.id))
    ).set((todo::dsl::content.eq(&todo.content), todo::dsl::expire_time.eq(&todo.expire_time)))
        .execute(conn).expect("Update error");
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