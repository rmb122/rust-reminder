use std::borrow::Borrow;
use std::cell::{Ref, RefCell};
use std::rc::Rc;

use chrono::{Date, DateTime, Local, NaiveDate, TimeZone, Utc};
use diesel::SqliteConnection;
use gtk::{Application, gdk_pixbuf, glib, IconSize, IconView, ListBox, ListBoxRow};
use gtk::prelude::*;

use crate::models::{db_new_todo, establish_connection, Todo};
use crate::utils::{get_border_label, get_icon_view, get_todo_row_view};

pub struct ResetDateButton {
    pub reset_date_btn: gtk::IconView,
    pub current_date_label: gtk::Label,
}

impl ResetDateButton {
    fn new() -> Self {
        ResetDateButton {
            reset_date_btn: get_icon_view(&["edit-undo"]).unwrap(),
            current_date_label: gtk::Label::new(None),
        }
    }

    fn hide(&self) {
        self.reset_date_btn.hide();
        self.current_date_label.hide();
    }

    fn show(&self) {
        self.reset_date_btn.show();
        self.current_date_label.show();
    }

    fn set_date(&self, date: Date<Local>) {
        self.current_date_label.set_label(&date.format("%Y-%m-%d").to_string());
    }
}

pub struct Reminder {
    pub db_conn: Rc<SqliteConnection>,
    pub todo_edit_panel_button: Vec<(&'static str, &'static dyn Fn(&Self))>,
    pub todo_msg_list: Rc<gtk::ListBox>,
    pub current_date: Rc<RefCell<Option<chrono::Date<Local>>>>,
    pub reset_date_btn: Rc<ResetDateButton>,
}

impl Clone for Reminder {
    fn clone(&self) -> Self {
        return Reminder {
            db_conn: Rc::clone(&self.db_conn),
            todo_edit_panel_button: self.todo_edit_panel_button.clone(),
            todo_msg_list: Rc::clone(&self.todo_msg_list),
            current_date: Rc::clone(&self.current_date),
            reset_date_btn: Rc::clone(&self.reset_date_btn),
        };
    }
}

impl Reminder {
    pub fn new() -> Reminder {
        return Reminder {
            db_conn: Rc::new(establish_connection(None)),
            todo_edit_panel_button: vec![
                ("list-add", &Reminder::todo_add_callback),
                ("list-remove", &Reminder::todo_remove_callback),
                ("document-page-setup", &Reminder::todo_edit_callback),
            ],
            todo_msg_list: Rc::new(gtk::ListBox::new()),
            current_date: Rc::new(RefCell::new(None)),
            reset_date_btn: Rc::new(ResetDateButton::new()),
        };
    }

    fn todo_add_callback(&self) {
        let todo = Todo {
            id: 0,
            content: &String::from("test"),
            expire_time: Some(Utc::now().naive_utc()),
        };
        let row = get_todo_row_view(&todo);
        db_new_todo(self.db_conn.borrow(), &todo);

        self.todo_msg_list.add(&row);
        self.todo_msg_list.show_all();
    }

    fn todo_remove_callback(&self) {
        println!("remove callback");
    }

    fn todo_edit_callback(&self) {
        println!("edit callback");
    }

    fn reset_date(&self) {
        *self.current_date.borrow_mut() = None;
    }

    pub fn build_ui(&self, application: &Application) {
        let window = gtk::ApplicationWindow::new(application);

        window.set_title("Reminder");
        window.set_border_width(10);
        window.set_position(gtk::WindowPosition::Mouse);
        window.set_type_hint(gtk::gdk::WindowTypeHint::Dialog);
        window.set_default_size(600, -1);

        let main_box = gtk::Box::new(gtk::Orientation::Horizontal, 3);
        let todo_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let panel_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        panel_box.set_halign(gtk::Align::Fill);

        let todo_msg_list: &ListBox = self.todo_msg_list.borrow();
        (*todo_msg_list).set_selection_mode(gtk::SelectionMode::Multiple);
        (*todo_msg_list).set_activate_on_single_click(false);

        let mut icon_list: Vec<&str> = Vec::new();
        for button in self.todo_edit_panel_button.iter() {
            icon_list.push(button.0);
        }

        let todo_edit_panel = get_icon_view(&icon_list).unwrap();

        // glib::clone! break type hinting, just do myself...
        let self_clone = self.clone();
        todo_edit_panel.connect_selection_changed(move |e| {
            let items = (*e).selected_items();
            for item in items.iter() {
                assert_eq!(item.indices().len(), 1);
                (self_clone.todo_edit_panel_button[item.indices()[0] as usize].1)(&self_clone);
            }
            (*e).unselect_all();
        });

        let calendar = gtk::Calendar::new();
        calendar.set_width_request(250);

        let self_clone = self.clone();
        calendar.connect_day_selected(move |x| {
            let date = Local.ymd(x.year(), x.month() as u32, x.day() as u32);
            *self_clone.current_date.borrow_mut() = Some(date);
            (*self_clone.reset_date_btn).set_date(date);
            (*self_clone.reset_date_btn).show();
        });

        let scrolled_window = gtk::ScrolledWindow::builder().build();
        scrolled_window.add(todo_msg_list);

        let reset_date_btn: &ResetDateButton = self.reset_date_btn.borrow();
        let reset_date_icon_view = &reset_date_btn.reset_date_btn;
        let reset_date_label = &reset_date_btn.current_date_label;

        let self_clone = self.clone();
        reset_date_icon_view.connect_selection_changed(move |e| {
            self_clone.reset_date();
            (*e).unselect_all();
            self_clone.reset_date_btn.hide();
        });

        (*reset_date_label).set_margin_start(3);

        panel_box.pack_start(reset_date_icon_view, false, false, 0);
        panel_box.pack_start(reset_date_label, false, false, 0);
        panel_box.pack_start(&gtk::Label::new(None), true, true, 0); // padding
        panel_box.pack_start(&todo_edit_panel, false, false, 0);

        todo_box.pack_start(&panel_box, false, false, 0);
        todo_box.pack_start(&scrolled_window, true, true, 0);

        main_box.pack_start(&todo_box, true, true, 0);
        main_box.pack_start(&calendar, false, true, 0);

        window.add(&main_box);
        window.show_all();

        reset_date_btn.hide(); // hide reset btn in default
    }
}