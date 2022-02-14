use std::fs::create_dir_all;
use std::path;

use chrono::{Date, Local, NaiveDateTime, TimeZone};
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
        Todo {
            id: self.id.clone(),
            content: self.content.clone(),
            expire_time: self.expire_time.clone(),
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
            let time_start = date.and_hms(0, 0, 0).naive_local();
            let time_end = date.and_hms(23, 59, 59).naive_local();

            todo::dsl::todo.filter(todo::dsl::expire_time.between(time_start, time_end)).order_by(todo::dsl::expire_time).load::<Todo>(conn).expect("Query error")
        }
        None => {
            todo::dsl::todo.filter(todo::dsl::expire_time.is_null()).order_by(todo::dsl::id).load::<Todo>(conn).expect("Query error")
        }
    }
}

pub fn db_del_todo(conn: &SqliteConnection, todo_id: &Vec<i32>) {
    if todo_id.len() <= 0 {
        return;
    }
    diesel::delete(todo::table.filter(todo::id.eq_any(todo_id))).execute(conn).expect("Delete error");
}

pub fn db_update_todo(conn: &SqliteConnection, todo: &Todo) {
    diesel::update(
        todo::table.filter(todo::dsl::id.eq(todo.id))
    ).set((todo::dsl::content.eq(&todo.content), todo::dsl::expire_time.eq(&todo.expire_time)))
        .execute(conn).expect("Update error");
}


pub fn db_get_exists_day(conn: &SqliteConnection, year: i32, month: i32) -> Vec<i32> {
    use diesel::sql_types::Text;
    use diesel::sql_types::Nullable;
    use diesel::sql_types::Timestamp;

    sql_function! {
        fn strftime(format: Text, time: Nullable<Timestamp>) -> Nullable<Text>;
    }

    let start_time = Local.ymd(year, month as u32, 1).and_hms(0, 0, 0).naive_local();

    let end_time = if month + 1 <= 12 {
        Local.ymd(year, (month + 1) as u32, 1).and_hms(0, 0, 0).naive_local()
    } else {
        Local.ymd(year + 1, 1 as u32, 1).and_hms(0, 0, 0).naive_local()
    };

    let days: Vec<Option<String>> = todo::dsl::todo.select(strftime("%d", todo::dsl::expire_time)).
        filter(todo::dsl::expire_time.is_not_null().and(todo::dsl::expire_time.between(start_time, end_time))).load(conn).expect("Get day error");

    return days.iter().map(|x| { x.as_ref().unwrap().parse::<i32>().unwrap() }).collect();
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