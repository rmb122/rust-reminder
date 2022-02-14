use chrono::{Local, TimeZone};
use gtk::{gdk_pixbuf, IconSize, pango};
use gtk::prelude::*;

use crate::models::Todo;

pub fn get_icon_view(icon_list: &[&str]) -> Option<gtk::IconView> {
    let icon_view = gtk::IconView::builder().
        item_padding(5).columns(icon_list.len() as i32).
        column_spacing(0).selection_mode(gtk::SelectionMode::Single).margin(0)
        .build();

    let col_types = [gdk_pixbuf::Pixbuf::static_type()];
    let model = gtk::ListStore::new(&col_types);

    let icon_theme = gtk::IconTheme::default()?;
    let (icon_size, _) = IconSize::Menu.lookup()?;

    for icon in icon_list.iter() {
        let icon = icon_theme.load_icon(icon, icon_size, gtk::IconLookupFlags::empty()).unwrap_or(None)?;

        model.insert_with_values(None, &[
            (0, &icon)
        ]);
    }

    icon_view.set_model(Some(&model));
    icon_view.set_pixbuf_column(0);

    Some(icon_view)
}

pub fn get_border_label(label_str: &str, markup: bool) -> gtk::Frame {
    let label = gtk::Label::builder().
        halign(gtk::Align::Start).margin_start(3).margin_end(3).wrap(true).wrap_mode(pango::WrapMode::WordChar).build();
    if markup {
        label.set_markup(label_str);
    } else {
        label.set_label(label_str);
    }

    let frame = gtk::Frame::new(None);
    frame.set_shadow_type(gtk::ShadowType::Out);
    frame.add(&label);
    return frame;
}

pub fn get_todo_row_view(todo: &Todo) -> gtk::Grid {
    let grid = gtk::Grid::new();

    let label = get_border_label(&todo.content, false);
    label.set_expand(true);
    grid.attach(&label, 1, 0, 1, 1);

    if todo.expire_time.is_some() {
        let label = get_border_label(&Local.from_local_datetime(&todo.expire_time.unwrap()).unwrap().format("%H:%M").to_string(), false);
        grid.attach(&label, 2, 0, 1, 1);
    }

    unsafe {
        grid.set_data("todo", todo.clone());
    }
    return grid;
}
