#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;


use gtk::prelude::*;

use crate::reminder::Reminder;

mod utils;
mod reminder;
mod models;
mod schema;
mod reminder_edit_dialog;

fn main() {
    let application =
        gtk::Application::new(Some("com.rmb122.reminder"), Default::default());
    application.connect_activate(|app| { Reminder::new().build_ui(app) });
    application.run();
}