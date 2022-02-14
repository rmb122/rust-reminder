use std::borrow::{Borrow, BorrowMut};
use std::cell::{Ref, RefCell};
use std::ops::Deref;
use std::rc::Rc;

use chrono::{Date, DateTime, Local, NaiveDate, TimeZone, Utc};
use diesel::SqliteConnection;
use gtk::{Application, ApplicationWindow, gdk_pixbuf, glib, IconSize, IconView, ListBox, ListBoxRow};
use gtk::prelude::*;

use crate::models::{db_del_todo, db_find_todo, db_new_todo, db_update_todo, establish_connection, NewTodo, Todo};
use crate::reminder_edit_dialog::ReminderEditDialog;
use crate::utils::{get_border_label, get_icon_view, get_todo_row_view};

pub struct ResetDateButton {
    reset_date_btn: gtk::IconView,
    current_date_label: gtk::Label,
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
    db_conn: Rc<SqliteConnection>,
    todo_edit_panel_button: Vec<(&'static str, &'static dyn Fn(&Self))>,
    todo_msg_list: Rc<gtk::ListBox>,
    current_date: Rc<RefCell<Option<Date<Local>>>>,
    reset_date_btn: Rc<ResetDateButton>,
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
        let date = self.current_date.deref().borrow().clone();

        let todo_add_dialog = ReminderEditDialog::new("New todo", date.is_some());
        if let Some(date) = date {
            todo_add_dialog.set_time(date.and_time(Local::now().time()).unwrap());
        }

        todo_add_dialog.show();

        let self_clone = self.clone();
        todo_add_dialog.connect_hide(move |save_todo, content, time| {
            if save_todo {
                let todo = match time {
                    Some(time) => {
                        NewTodo {
                            content: content,
                            expire_time: Some(time.naive_utc()),
                        }
                    }
                    None => {
                        NewTodo {
                            content: content,
                            expire_time: None,
                        }
                    }
                };

                db_new_todo(self_clone.db_conn.borrow(), &todo);
                self_clone.todo_refresh()
            }
        });
    }

    fn todo_remove_callback(&self) {
        let mut todo_id = Vec::<i32>::new();
        self.todo_msg_list.selected_foreach(|_, r| unsafe {
            if let Some(todo) = r.child().unwrap().data::<Todo>("todo") {
                todo_id.push(todo.as_ref().id);

            }
        });

        db_del_todo(self.db_conn.deref(), &todo_id);
        self.todo_refresh();
    }

    fn todo_edit_callback(&self) {
        let row = self.todo_msg_list.selected_row();
        if row.is_none() {
            return;
        }

        let row = row.unwrap();
        let todo = unsafe {
            if let Some(todo) = row.child().unwrap().data::<Todo>("todo") {
                Some(todo.as_ref().clone())
            } else {
                None
            }
        };

        if todo.is_none() {
            return;
        }
        let todo = todo.unwrap();

        let todo_add_dialog = ReminderEditDialog::new("Edit todo", todo.expire_time.is_some());
        if let Some(time) = todo.expire_time {
            todo_add_dialog.set_time(Local.from_utc_datetime(&time));
        }

        todo_add_dialog.set_content(todo.content);
        todo_add_dialog.show();

        let self_clone = self.clone();
        todo_add_dialog.connect_hide(move |save_todo, content, time| {
            if save_todo {
                let todo = match time {
                    Some(time) => {
                        Todo {
                            id: todo.id,
                            content: content,
                            expire_time: Some(time.naive_utc()),
                        }
                    }
                    None => {
                        Todo {
                            id: todo.id,
                            content: content,
                            expire_time: None,
                        }
                    }
                };

                db_update_todo(self_clone.db_conn.borrow(), &todo);
                self_clone.todo_refresh()
            }
        });
    }

    fn todo_refresh(&self) {
        self.todo_msg_list.foreach(|r| {
            self.todo_msg_list.remove(r);
        }); // clear list items

        let todo_list = match self.current_date.deref().borrow().deref() {
            Some(date) => {
                db_find_todo(self.db_conn.deref(), Some(date.clone()))
            }
            None => {
                db_find_todo(self.db_conn.deref(), None)
            }
        };

        for todo in todo_list.iter() {
            let todo = get_todo_row_view(todo);
            self.todo_msg_list.add(&todo);
        }
        self.todo_msg_list.show_all();
    }

    fn reset_date(&self) {
        *self.current_date.deref().borrow_mut() = None;
    }

    pub fn build_ui(&self, application: &Application) {
        let window = gtk::ApplicationWindow::builder().
            application(application).title("Reminder").border_width(10).
            window_position(gtk::WindowPosition::Mouse).type_hint(gtk::gdk::WindowTypeHint::Dialog).
            default_width(600).default_height(-1).build();

        let main_box = gtk::Box::new(gtk::Orientation::Horizontal, 3);
        let todo_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let panel_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        panel_box.set_halign(gtk::Align::Fill);

        let todo_msg_list: &ListBox = self.todo_msg_list.borrow();
        todo_msg_list.set_selection_mode(gtk::SelectionMode::Multiple);
        todo_msg_list.set_activate_on_single_click(false);

        let mut icon_list: Vec<&str> = Vec::new();
        for button in self.todo_edit_panel_button.iter() {
            icon_list.push(button.0);
        }

        let todo_edit_panel = get_icon_view(&icon_list).unwrap();

        // glib::clone! break type hinting, just do it myself...
        let self_clone = self.clone();
        todo_edit_panel.connect_selection_changed(move |e| {
            let items = e.selected_items();
            for item in items.iter() {
                assert_eq!(item.indices().len(), 1);
                (self_clone.todo_edit_panel_button[item.indices()[0] as usize].1)(&self_clone);
            }
            e.unselect_all();
        });

        let calendar = gtk::Calendar::new();
        calendar.set_width_request(250);

        let self_clone = self.clone();
        calendar.connect_day_selected(move |x| {
            let date = Local.ymd(x.year(), (x.month() + 1) as u32, x.day() as u32);
            *self_clone.current_date.deref().borrow_mut() = Some(date);
            self_clone.reset_date_btn.set_date(date);
            self_clone.reset_date_btn.show();
            self_clone.todo_refresh();
        });

        let scrolled_window = gtk::ScrolledWindow::builder().build();
        scrolled_window.add(todo_msg_list);

        let reset_date_btn: &ResetDateButton = self.reset_date_btn.borrow();
        let reset_date_icon_view = &reset_date_btn.reset_date_btn;
        let reset_date_label = &reset_date_btn.current_date_label;

        let self_clone = self.clone();
        reset_date_icon_view.connect_selection_changed(move |e| {
            self_clone.reset_date();
            e.unselect_all();
            self_clone.reset_date_btn.hide();
            self_clone.todo_refresh();
        });

        reset_date_label.set_margin_start(3);

        panel_box.pack_start(reset_date_icon_view, false, false, 0);
        panel_box.pack_start(reset_date_label, false, false, 0);
        panel_box.pack_start(&gtk::Label::new(None), true, true, 0); // padding
        panel_box.pack_start(&todo_edit_panel, false, false, 0);

        todo_box.pack_start(&panel_box, false, false, 0);
        todo_box.pack_start(&scrolled_window, true, true, 0);

        main_box.pack_start(&todo_box, true, true, 0);
        main_box.pack_start(&calendar, false, true, 0);

        self.todo_refresh(); // get todo list
        window.add(&main_box);
        window.show_all();

        reset_date_btn.hide(); // hide reset btn in default
    }
}