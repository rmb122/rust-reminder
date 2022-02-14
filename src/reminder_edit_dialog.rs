use std::borrow::Borrow;
use std::cell::{Ref, RefCell};
use std::ops::Deref;
use std::rc::Rc;
use std::str::FromStr;

use chrono::{Datelike, DateTime, Local, Timelike, TimeZone};
use gtk::{Application, Container, CssProvider, Dialog, StateFlags, TextBuffer, Widget};
use gtk::prelude::*;

pub struct Timepicker {
    year_picker: gtk::SpinButton,
    month_picker: gtk::SpinButton,
    day_picker: gtk::SpinButton,
    hour_picker: gtk::SpinButton,
    minute_picker: gtk::SpinButton,
}

impl Timepicker {
    fn new() -> Self {
        let todo_year_picker = gtk::SpinButton::builder().adjustment(
            &gtk::Adjustment::builder().upper(2077f64).lower(1970f64).step_increment(1f64).build()
        ).orientation(gtk::Orientation::Horizontal).build();
        let todo_month_picker = gtk::SpinButton::builder().adjustment(
            &gtk::Adjustment::builder().upper(12f64).lower(1f64).step_increment(1f64).build()
        ).orientation(gtk::Orientation::Horizontal).build();
        let todo_day_picker = gtk::SpinButton::builder().adjustment(
            &gtk::Adjustment::builder().upper(31f64).lower(1f64).step_increment(1f64).build()
        ).orientation(gtk::Orientation::Horizontal).build();

        let todo_hour_picker = gtk::SpinButton::builder().wrap(true).adjustment(
            &gtk::Adjustment::builder().upper(23f64).lower(0f64).step_increment(1f64).build()
        ).orientation(gtk::Orientation::Vertical).build();
        let todo_minute_picker = gtk::SpinButton::builder().wrap(true).adjustment(
            &gtk::Adjustment::builder().upper(59f64).lower(0f64).step_increment(1f64).build()
        ).orientation(gtk::Orientation::Vertical).build();
        todo_minute_picker.connect_output(|x| {
            x.set_text(&format!("{:02}", x.value()));
            return gtk::Inhibit(true);
        });

        Timepicker {
            year_picker: todo_year_picker,
            month_picker: todo_month_picker,
            day_picker: todo_day_picker,
            hour_picker: todo_hour_picker,
            minute_picker: todo_minute_picker,
        }
    }

    fn build_ui(&self) -> impl IsA<Widget> {
        let todo_timepicker = gtk::Box::builder().orientation(gtk::Orientation::Horizontal).
            halign(gtk::Align::Center).spacing(3).margin_start(3).margin_bottom(6).build();

        let todo_date_timepicker = gtk::Grid::builder().margin_start(3).build();

        todo_date_timepicker.attach(&gtk::Label::builder().label("Year").margin_end(6).halign(gtk::Align::End).build(), 0, 0, 1, 1);
        todo_date_timepicker.attach(&self.year_picker, 1, 0, 1, 1);
        todo_date_timepicker.attach(&gtk::Label::builder().label("Month").margin_end(6).halign(gtk::Align::End).build(), 0, 1, 1, 1);
        todo_date_timepicker.attach(&self.month_picker, 1, 1, 1, 1);
        todo_date_timepicker.attach(&gtk::Label::builder().label("Day").margin_end(6).halign(gtk::Align::End).build(), 0, 2, 1, 1);
        todo_date_timepicker.attach(&self.day_picker, 1, 2, 1, 1);

        let todo_time_timepicker = gtk::Box::builder().orientation(gtk::Orientation::Horizontal).spacing(3).margin_start(6).build();

        todo_time_timepicker.pack_start(&self.hour_picker, false, false, 0);
        todo_time_timepicker.pack_start(&gtk::Label::builder().label(":").build(), false, false, 0);
        todo_time_timepicker.pack_start(&self.minute_picker, false, false, 0);

        todo_timepicker.pack_start(&todo_date_timepicker, true, true, 0);
        todo_timepicker.pack_start(&todo_time_timepicker, true, true, 0);

        let todo_timepicker_with_label = gtk::Box::builder().orientation(gtk::Orientation::Vertical).
            halign(gtk::Align::Fill).build();
        let label = gtk::Label::builder().label("<b>DateTime:</b>").use_markup(true).margin_start(3).halign(gtk::Align::Start).build();
        todo_timepicker_with_label.pack_start(&label, false, false, 0);
        todo_timepicker_with_label.pack_start(&todo_timepicker, false, false, 0);

        return todo_timepicker_with_label;
    }

    fn set_time(&self, time: DateTime<Local>) {
        self.year_picker.set_value(time.year() as f64);
        self.month_picker.set_value(time.month() as f64);
        self.day_picker.set_value(time.day() as f64);
        self.hour_picker.set_value(time.hour() as f64);
        self.minute_picker.set_value(time.minute() as f64);
    }

    fn get_time(&self) -> DateTime<Local> {
        Local.ymd(self.year_picker.value() as i32, self.month_picker.value() as u32, self.day_picker.value() as u32)
            .and_hms(self.hour_picker.value() as u32, self.minute_picker.value() as u32, 0u32)
    }
}

pub struct ReminderEditDialog {
    dialog: Rc<gtk::Dialog>,
    todo_content_view: Rc<gtk::TextView>,
    todo_timepicker: Rc<Option<Timepicker>>,
    save_todo: Rc<RefCell<bool>>,
}

impl Clone for ReminderEditDialog {
    fn clone(&self) -> Self {
        ReminderEditDialog {
            dialog: Rc::clone(&self.dialog),
            todo_content_view: Rc::clone(&self.todo_content_view),
            todo_timepicker: Rc::clone(&self.todo_timepicker),
            save_todo: Rc::clone(&self.save_todo),
        }
    }
}

impl ReminderEditDialog {
    pub fn new(title: &str, have_timepicker: bool) -> Self {
        let dialog = gtk::Dialog::builder().height_request(150).type_hint(gtk::gdk::WindowTypeHint::Dialog)
            .width_request(400).title(title).destroy_with_parent(true).build();

        let content_label = gtk::Label::builder().margin_start(3).label("<b>Content:</b>").use_markup(true).halign(gtk::Align::Start).build();

        let todo_content_window = gtk::ScrolledWindow::builder().border_width(3).build();
        let todo_content_frame = gtk::Frame::builder().border_width(3).build();
        let todo_content_view = gtk::TextView::builder().wrap_mode(gtk::WrapMode::WordChar).build();
        todo_content_window.add(&todo_content_view);
        todo_content_frame.add(&todo_content_window);

        let button_box = gtk::Box::builder().orientation(gtk::Orientation::Horizontal).halign(gtk::Align::Center).margin_bottom(6).margin_top(3).build();
        let cancel_button = gtk::Button::builder().label("Cancel").margin_end(3).build();
        let save_button = gtk::Button::builder().label("Save").margin_start(3).build();

        button_box.pack_start(&cancel_button, false, false, 0);
        button_box.pack_start(&save_button, false, false, 0);

        dialog.content_area().pack_start(&content_label, false, false, 0);
        dialog.content_area().pack_start(&todo_content_frame, true, true, 0);

        let mut time_picker = None;
        if have_timepicker {
            let real_time_picker = Timepicker::new();
            dialog.content_area().pack_start(&real_time_picker.build_ui(), false, false, 0);
            time_picker = Some(real_time_picker);
        }

        dialog.content_area().pack_start(&button_box, false, false, 0);

        let dialog = ReminderEditDialog {
            dialog: Rc::new(dialog),
            todo_content_view: Rc::new(todo_content_view),
            todo_timepicker: Rc::new(time_picker),
            save_todo: Rc::new(RefCell::new(false)),
        };

        let self_clone = dialog.clone();
        cancel_button.connect_clicked(move |x| {
            *self_clone.save_todo.deref().borrow_mut() = false;
            self_clone.dialog.hide();
        });

        let self_clone = dialog.clone();
        save_button.connect_clicked(move |x| {
            *self_clone.save_todo.deref().borrow_mut() = true;
            self_clone.dialog.hide();
        });

        let self_clone = dialog.clone();
        dialog.dialog.connect_key_press_event(move |_, e| {
            if e.keyval() == gtk::gdk::keys::constants::Return {
                *self_clone.save_todo.deref().borrow_mut() = true;
                self_clone.dialog.hide();
            }
            return gtk::Inhibit(false);
        });

        return dialog;
    }

    pub fn show(&self) {
        self.dialog.show_all();
    }

    pub fn set_time(&self, time: DateTime<Local>) {
        let todo_timepicker: &Option<Timepicker> = self.todo_timepicker.borrow();
        match todo_timepicker {
            Some(todo_timepicker) => {
                todo_timepicker.set_time(time);
            }
            None => {}
        };
    }

    pub fn set_content(&self, content: String) {
        let buffer = gtk::TextBuffer::builder().text(&content).build();
        self.todo_content_view.set_buffer(Some(&buffer));
    }

    pub fn get_time(&self) -> Option<DateTime<Local>> {
        let todo_timepicker: &Option<Timepicker> = self.todo_timepicker.borrow();
        match todo_timepicker {
            Some(todo_timepicker) => {
                Some(todo_timepicker.get_time())
            }
            None => {
                None
            }
        }
    }

    pub fn connect_hide<F: 'static>(&self, f: F) where F: Fn(bool, String, Option<DateTime<Local>>) {
        let self_clone = self.clone();
        self.dialog.connect_hide(move |_| {
            let todo_timepicker: &Option<Timepicker> = self_clone.todo_timepicker.borrow();
            let time = match todo_timepicker {
                Some(todo_timepicker) => {
                    Some(todo_timepicker.get_time())
                }
                None => {
                    None
                }
            };

            let text = match self_clone.todo_content_view.buffer() {
                Some(buffer) => {
                    let (start, end) = buffer.bounds();

                    match buffer.text(&start, &end, false) {
                        Some(text) => {
                            String::from(text.as_str())
                        }
                        None => {
                            String::new()
                        }
                    }
                }
                None => {
                    String::new()
                }
            };

            f(*self_clone.save_todo.borrow_mut(), text, time);
        });
    }
}