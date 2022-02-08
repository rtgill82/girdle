//
// Copyright (c) 2022, Robert Gill <rtgill82@gmail.com>
//

use std::process;
use std::borrow::Borrow;
use std::cell::{Ref,RefCell};
use std::rc::Rc;

use gdk;
use glib::signal;
use glib::SignalHandlerId;
use gtk::prelude::*;

use crate::Dictionary;
use crate::dictionary::Error;
use crate::dictionary::SetType;

use crate::DICTIONARIES;

struct DeleteSignalIds {
    exclude: RefCell<Option<SignalHandlerId>>,
    include: RefCell<Option<SignalHandlerId>>
}

pub struct UI {
    dictionary: Dictionary,
    signal_ids: DeleteSignalIds,
    application: gtk::Application,
    include: gtk::Entry,
    exclude: gtk::Entry,
    results: gtk::TextView,
    positions: [gtk::Entry; 5]
}

impl DeleteSignalIds {
    pub fn new() -> DeleteSignalIds {
        DeleteSignalIds {
            exclude: RefCell::new(None),
            include: RefCell::new(None)
        }
    }

    pub fn signal(&self, hook_type: SetType) -> Ref<SignalHandlerId> {
        match hook_type {
            SetType::Excluded => Ref::map(self.exclude.borrow(), |v|
                                          v.as_ref().unwrap()),
            SetType::Included => Ref::map(self.include.borrow(), |v|
                                          v.as_ref().unwrap())
        }
    }
}

impl UI {
    pub fn run(id: &str) {
        gtk::init().expect("Cannot initialize GTK.");

        let result = Dictionary::new(DICTIONARIES);
        if let Err(error) = result {
            show_error_dialog(id, error);
        }

        let ui = new_ui(id, result.unwrap());
        let include = connect_delete_text(SetType::Included, &ui);
        let exclude = connect_delete_text(SetType::Excluded, &ui);
        ui.set_signal_ids(include, exclude);

        connect_focus_out_event(SetType::Included, &ui);
        connect_insert_text(SetType::Included, &ui);

        connect_focus_out_event(SetType::Excluded, &ui);
        connect_insert_text(SetType::Excluded, &ui);

        position_connect_delete_text(&ui);
        position_connect_focus_out_event(&ui);
        position_connect_insert_text(&ui);
        application_connect_activate(&ui);
        ui.application.run();
    }

    fn refresh(&self) {
        let chars = self.dictionary.excluded_chars();
        let mut excluded = String::new();
        for ch in chars.iter() { excluded.push(*ch); }

        let chars = self.dictionary.included_chars();
        let mut included = String::new();
        for ch in chars.iter() { included.push(*ch); }

        let signal_id = self.signal_ids.signal(SetType::Excluded);
        self.exclude.block_signal(&signal_id);
        self.exclude.set_text(&excluded);
        self.exclude.unblock_signal(&signal_id);

        let signal_id = self.signal_ids.signal(SetType::Included);
        self.include.block_signal(&signal_id);
        self.include.set_text(&included);
        self.include.unblock_signal(&signal_id);
    }

    fn set_signal_ids(&self, include: SignalHandlerId,
                             exclude: SignalHandlerId)
    {
        *self.signal_ids.include.borrow_mut() = Some(include);
        *self.signal_ids.exclude.borrow_mut() = Some(exclude);
    }
}

fn show_error_dialog(id: &str, error: Error) -> ! {
    let application = gtk::Application::new(Some(id), Default::default());
    application.connect_activate(move |app| {
        let dialog = gtk::MessageDialog::builder()
            .buttons(gtk::ButtonsType::Ok)
            .message_type(gtk::MessageType::Error)
            .text(&format!("{}", error))
            .application(app)
            .modal(true)
            .title("Error")
            .window_position(gtk::WindowPosition::CenterAlways)
            .build();

        dialog.connect_response(move |dialog, _| {
            unsafe { dialog.destroy(); }
        });
        dialog.show_all();
    });
    application.run();
    process::exit(1);
}

fn new_ui(id: &str, dictionary: Dictionary) -> Rc<UI> {
    let application = gtk::Application::new(Some(id), Default::default());

    let mut vec = Vec::new();
    for i in 0usize..5 {
        let entry = gtk::Entry::new();
        entry.set_max_length(1);
        unsafe { entry.set_data("index", i); }
        vec.push(entry);
    }

    let results = gtk::TextView::new();
    results.set_cursor_visible(false);
    results.set_editable(false);

    let ui = UI {
        dictionary: dictionary,
        application: application,
        include: gtk::Entry::new(),
        exclude: gtk::Entry::new(),
        results: results,
        signal_ids: DeleteSignalIds::new(),
        positions: vec.try_into().unwrap()
    };

    Rc::new(ui)
}

fn build_menubar(ui: &Rc<UI>) -> gtk::MenuBar {
    let menubar = gtk::MenuBar::new();
    let file_menu = gtk::Menu::new();

    let file = gtk::MenuItem::with_mnemonic("_File");
    let reset = gtk::MenuItem::with_mnemonic("_Reset");
    let quit = gtk::MenuItem::with_mnemonic("_Quit");

    let ui_ptr = Rc::downgrade(ui);
    reset.connect_activate(move |_| {
        let rc = ui_ptr.upgrade().unwrap();
        let ui: &UI = rc.borrow();

        ui.dictionary.reset();
        ui.refresh();

        for pos in 0..5 {
            ui.positions[pos].set_text("");
        }

        let buffer = ui.results.buffer()
            .expect("Couldn't get results buffer.");
        buffer.set_text("");
    });

    let ui_ptr = Rc::downgrade(ui);
    quit.connect_activate(move |_| {
        let rc = ui_ptr.upgrade().unwrap();
        let ui: &UI = rc.borrow();

        ui.application.quit()
    });

    file.set_submenu(Some(&file_menu));
    file_menu.append(&reset);
    file_menu.append(&quit);
    menubar.add(&file);

    menubar
}

fn build_ui(ui: &Rc<UI>) -> gtk::Box {
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let menubar = build_menubar(ui);
    vbox.add(&menubar);

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let include = build_character_entry("Correct Characters", &ui.include);
    hbox.pack_start(&include, true, true, 0);
    let exclude = build_character_entry("Incorrect Characters", &ui.exclude);
    hbox.pack_start(&exclude, true, true, 0);
    vbox.add(&hbox);

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let label = gtk::Label::new(Some("Exact Positions"));
    hbox.add(&label);
    vbox.add(&hbox);

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    for i in 0..5 {
        hbox.pack_start(&ui.positions[i], true, false, 0);
    }
    vbox.add(&hbox);

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let label = gtk::Label::new(Some("Results"));

    hbox.add(&label);
    vbox.add(&hbox);

    let none = gtk::Adjustment::NONE;
    let window = gtk::ScrolledWindow::new(none, none);
    window.set_shadow_type(gtk::ShadowType::In);
    window.add(&ui.results);
    vbox.pack_start(&window, true, true, 0);

    return vbox;
}

fn build_character_entry(label: &str, entry: &gtk::Entry) -> gtk::Box {
    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let label = gtk::Label::new(Some(label));
    vbox.add(&hbox);
    hbox.add(&label);
    vbox.add(entry);
    vbox
}

fn application_connect_activate(ui: &Rc<UI>) {
    let vbox = build_ui(&ui);
    ui.application.connect_activate(move |app| {
        let window = gtk::ApplicationWindow::new(app);

        window.set_title("Girdle");
        window.set_border_width(8);
        window.set_position(gtk::WindowPosition::Center);
        window.add(&vbox);
        window.show_all();
    });
}

fn is_non_include_character(ch: char) -> bool {
    !(ch.is_ascii_alphabetic() || ch == ',' || ch == ' ')
}

fn display_results(dict: &Dictionary, results: &gtk::TextView) {
    let matches =  dict.matches();
    let buffer = results.buffer()
        .expect("Couldn't get results buffer.");

    match &*matches {
        Some(matches) => {
            let mut results = String::new();
            for word in &*matches {
                let s = format!("{}\n", word);
                results.push_str(&s);
            }
            buffer.set_text(&results);
        },

        None => {
            buffer.set_text("")
        }
    }
}

fn connect_delete_text(hook_type: SetType, ui: &Rc<UI>) -> SignalHandlerId {
    let entry = match hook_type {
        SetType::Excluded => &ui.exclude,
        SetType::Included => &ui.include
    };

    let ui_ptr = Rc::downgrade(ui);
    let id = entry.connect_delete_text(move |entry, start, end| {
        let rc = ui_ptr.upgrade().unwrap();
        let ui: &UI = rc.borrow();

        let gstring = entry.text();
        let s = gstring.as_str();
        let start: usize = start.try_into().unwrap();
        let end: usize = end.try_into().unwrap();

        for ch in s[start..end].chars() {
            ui.dictionary.remove_char(hook_type, ch);
        }
        display_results(&ui.dictionary, &ui.results);
    });

    return id;
}

fn connect_focus_out_event(hook_type: SetType, ui: &Rc<UI>) {
    let entry = match hook_type {
        SetType::Excluded => &ui.exclude,
        SetType::Included => &ui.include
    };

    let ui_ptr = Rc::downgrade(ui);
    entry.connect_focus_out_event(move |entry, _| {
        let rc = ui_ptr.upgrade().unwrap();
        let ui: &UI = rc.borrow();

        ui.dictionary.clear_set(hook_type);

        let gstring = entry.text();
        let text = gstring.as_str();
        for ch in text.chars() {
            if ch.is_ascii_alphabetic() {
                ui.dictionary.add_char(hook_type, ch);
            }
        }

        ui.refresh();
        Inhibit(false)
    });
}

fn connect_insert_text(hook_type: SetType, ui: &Rc<UI>) {
    let entry = match hook_type {
        SetType::Excluded => &ui.exclude,
        SetType::Included => &ui.include
    };

    let ui_ptr = Rc::downgrade(&ui);
    entry.connect_insert_text(move |entry, s, _| {
        if let Some(ch) = s.chars().next() {
            if is_non_include_character(ch) {
                gdk::beep();
                signal::signal_stop_emission_by_name(entry, "insert-text");
                return;
            }

            if ch.is_ascii_alphabetic() {
                let rc = ui_ptr.upgrade().unwrap();
                let ui: &UI = rc.borrow();
                ui.dictionary.add_char(hook_type, ch);
                display_results(&ui.dictionary, &ui.results);
            }
        }
    });
}

fn position_connect_delete_text(ui: &Rc<UI>) {
    for (pos, entry) in ui.positions.iter().enumerate() {
        let ui_ptr = Rc::downgrade(ui);
        entry.connect_delete_text(move |_, _, _| {
            let rc = ui_ptr.upgrade().unwrap();
            let ui: &UI = rc.borrow();

            ui.dictionary.unset_char_position(pos+1);
            display_results(&ui.dictionary, &ui.results);
        });
    }
}

fn position_connect_insert_text(ui: &Rc<UI>) {
    for entry in &ui.positions {
        let ui_ptr = Rc::downgrade(ui);
        entry.connect_insert_text(move |entry, s, pos| {
            if *pos > 0 { return; }

            if let Some(ch) = s.chars().next() {
                if !ch.is_ascii_alphabetic() {
                    gdk::beep();
                    signal::signal_stop_emission_by_name(entry, "insert-text");
                    entry.set_text("");
                    return;
                }

                let rc = ui_ptr.upgrade().unwrap();
                let ui: &UI = rc.borrow();
                let pos: usize = unsafe {
                    *entry.data("index").unwrap().as_ptr()
                };
                ui.dictionary.set_char_position(pos+1, ch);
                display_results(&ui.dictionary, &ui.results);
            }
        });
    }
}

fn position_connect_focus_out_event(ui: &Rc<UI>) {
    for entry in &ui.positions {
        let ui_ptr = Rc::downgrade(ui);

        entry.connect_focus_out_event(move |_, _| {
            let rc = ui_ptr.upgrade().unwrap();
            let ui: &UI = rc.borrow();

            ui.refresh();
            Inhibit(false)
        });
    }
}
