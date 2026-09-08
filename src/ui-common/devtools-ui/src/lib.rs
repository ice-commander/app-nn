mod i18n;
mod tool_page;

use gtk::prelude::*;
use i18n::tr;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

pub fn tr_tooltip() -> String {
    tr("devtools.toolbar_tooltip")
}

pub fn title_of(id: &str) -> String {
    tr(&format!("devtools.{id}"))
}

pub fn show_dialog(parent: &gtk::Window, seed: Option<PathBuf>) {
    let window = gtk::Window::builder()
        .title(tr("devtools.window_title"))
        .transient_for(parent)
        .modal(true)
        .default_width(900)
        .default_height(600)
        .resizable(true)
        .build();

    let main_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);

    let sidebar = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .width_request(200)
        .build();

    let search = gtk::SearchEntry::builder()
        .placeholder_text(tr("devtools.search_placeholder"))
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .build();
    sidebar.append(&search);

    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::Single)
        .build();
    list.add_css_class("navigation-sidebar");
    sidebar.append(
        &gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&list)
            .vexpand(true)
            .build(),
    );

    let stack = gtk::Stack::builder()
        .transition_type(gtk::StackTransitionType::Crossfade)
        .hexpand(true)
        .vexpand(true)
        .build();

    let seed = Rc::new(seed);
    for (index, spec) in devtools_core::TOOLS.iter().enumerate() {
        let title = title_of(spec.id);
        let row = gtk::ListBoxRow::new();
        row.set_widget_name(spec.id);
        let label = gtk::Label::builder()
            .label(&title)
            .xalign(0.0)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(12)
            .margin_end(12)
            .build();
        row.set_child(Some(&label));
        list.append(&row);
        stack.add_named(&tool_page::build(spec, seed.clone()), Some(&format!("page_{index}")));
    }

    let stack_select = stack.clone();
    list.connect_row_selected(move |_, row| {
        if let Some(row) = row {
            stack_select.set_visible_child_name(&format!("page_{}", row.index()));
        }
    });

    let query: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
    let query_filter = query.clone();
    list.set_filter_func(move |row| {
        let id = row.widget_name().to_string();
        match devtools_core::spec(&id) {
            Some(spec) => devtools_core::search::matches(&query_filter.borrow(), spec, &title_of(&id)),
            None => true,
        }
    });
    let list_search = list.clone();
    search.connect_search_changed(move |entry| {
        *query.borrow_mut() = entry.text().to_string();
        list_search.invalidate_filter();
        if list_search.selected_row().is_none() {
            select_first_visible(&list_search);
        }
    });

    main_box.append(&sidebar);
    main_box.append(&gtk::Separator::new(gtk::Orientation::Vertical));
    main_box.append(&stack);
    window.set_child(Some(&main_box));

    select_first_visible(&list);

    let escape = gtk::EventControllerKey::new();
    let closing = window.clone();
    escape.connect_key_pressed(move |_, key, _, _| {
        if key == gtk::gdk::Key::Escape {
            closing.close();
            return gtk::glib::Propagation::Stop;
        }
        gtk::glib::Propagation::Proceed
    });
    window.add_controller(escape);

    window.present();
}

fn select_first_visible(list: &gtk::ListBox) {
    let mut index = 0;
    while let Some(row) = list.row_at_index(index) {
        if row.is_visible() {
            list.select_row(Some(&row));
            return;
        }
        index += 1;
    }
}
