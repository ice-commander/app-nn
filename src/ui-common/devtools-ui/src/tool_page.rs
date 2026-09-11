use crate::i18n::tr;
use devtools_core::{Group, Shape, ToolSpec};
use gtk::prelude::*;
use std::path::PathBuf;
use std::rc::Rc;

pub fn build(spec: &'static ToolSpec, seed: Rc<Option<PathBuf>>) -> gtk::Box {
    let page = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let input = text_area(tr("devtools.input"));
    let file = file_switch(spec, &seed);
    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let size = gtk::Label::builder().xalign(0.0).build();
    size.add_css_class("dim-label");
    controls.append(&size);
    if let Some(button) = &file {
        controls.append(button);
    }

    let status = gtk::Label::builder().xalign(0.0).wrap(true).build();
    status.add_css_class("dim-label");

    let result = Result_::new(spec.shape);
    let copy = gtk::Button::from_icon_name("edit-copy-symbolic");
    copy.set_tooltip_text(Some(&tr("devtools.copy")));
    copy.set_halign(gtk::Align::End);
    copy.set_hexpand(true);
    controls.append(&copy);

    page.append(&input.frame);
    page.append(&controls);
    page.append(&result.frame);
    if let Some(row) = &result.expected_row {
        page.append(row);
    }
    page.append(&status);

    let seed_for_run = seed.clone();
    let recompute = {
        let input = input.clone();
        let result = result.clone();
        let status = status.clone();
        let size = size.clone();
        let file = file.clone();
        move || {
            let path = match (&file, seed_for_run.as_ref()) {
                (Some(button), Some(path)) if button.is_active() => Some(path.clone()),
                _ => None,
            };
            match read_source(&input.text(), path.as_deref()) {
                Ok(bytes) => {
                    size.set_text(&format!("{} B", bytes.len()));
                    match run_tool(spec, &bytes) {
                        Ok(text) => {
                            result.set_text(&text);
                            status.set_text(&result.verdict());
                        }
                        Err(e) => {
                            result.set_text("");
                            status.set_text(&e);
                        }
                    }
                }
                Err(e) => {
                    size.set_text("");
                    result.set_text("");
                    status.set_text(&e);
                }
            }
        }
    };

    let on_typing = recompute.clone();
    input.view.buffer().connect_changed(move |_| on_typing());
    if let Some(button) = &file {
        let on_toggle = recompute.clone();
        button.connect_toggled(move |_| on_toggle());
    }
    if let Some(expected) = &result.expected {
        let result_cmp = result.clone();
        let status_cmp = status.clone();
        expected.connect_changed(move |_| status_cmp.set_text(&result_cmp.verdict()));
    }

    let taken = result.clone();
    copy.connect_clicked(move |btn| {
        if let Some(display) = gtk::gdk::Display::default() {
            display.clipboard().set_text(&taken.text());
            btn.set_tooltip_text(Some(&tr("devtools.copied")));
        }
    });

    page
}

fn read_source(typed: &str, file: Option<&std::path::Path>) -> Result<Vec<u8>, String> {
    match file {
        Some(path) => std::fs::read(path).map_err(|e| format!("{}: {e}", path.display())),
        None => Ok(typed.as_bytes().to_vec()),
    }
}

fn run_tool(spec: &ToolSpec, bytes: &[u8]) -> Result<String, String> {
    use devtools_core::{base64_tool, hash, json};

    let text = || String::from_utf8_lossy(bytes).to_string();

    match spec.group {
        Group::Hash => {
            let algo = hash::Algo::from_id(spec.id).ok_or("unknown hash")?;
            Ok(hash::digest(algo, bytes))
        }
        Group::Encoding => match spec.id {
            "encoding.base64_encode" => Ok(base64_tool::encode(base64_tool::Alphabet::Standard, bytes)),
            "encoding.hex_encode" => Ok(base64_tool::to_hex(bytes)),
            "encoding.base64_decode" => as_text(base64_tool::decode(&text())?),
            "encoding.hex_decode" => as_text(base64_tool::from_hex(&text())?),
            other => Err(format!("unknown encoder {other}")),
        },
        Group::Format => match spec.id {
            "format.json_pretty" => json::format(&text(), 2),
            "format.json_minify" => json::minify(&text()),
            other => Err(format!("unknown formatter {other}")),
        },
    }
}

fn as_text(raw: Vec<u8>) -> Result<String, String> {
    String::from_utf8(raw).map_err(|_| tr("devtools.not_text"))
}

fn file_switch(spec: &ToolSpec, seed: &Rc<Option<PathBuf>>) -> Option<gtk::CheckButton> {
    let path = seed.as_ref().as_ref().filter(|_| spec.takes_file)?;
    let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    Some(gtk::CheckButton::with_label(&format!("{} {name}", tr("devtools.use_file"))))
}

#[derive(Clone)]
struct Result_ {
    frame: gtk::Box,
    line: Option<gtk::Entry>,
    area: Option<TextArea>,
    expected: Option<gtk::Entry>,
    expected_row: Option<gtk::Box>,
}

impl Result_ {
    fn new(shape: Shape) -> Result_ {
        match shape {
            Shape::OneLine => {
                let frame = gtk::Box::new(gtk::Orientation::Vertical, 4);
                frame.append(&caption(tr("devtools.output")));
                let line = gtk::Entry::builder().editable(false).hexpand(true).build();
                line.add_css_class("monospace");
                frame.append(&line);

                let expected = gtk::Entry::builder()
                    .placeholder_text(tr("devtools.expected_placeholder"))
                    .hexpand(true)
                    .build();
                expected.add_css_class("monospace");
                let compare = gtk::Box::new(gtk::Orientation::Vertical, 4);
                compare.append(&caption(tr("devtools.expected")));
                compare.append(&expected);

                Result_ {
                    frame,
                    line: Some(line),
                    area: None,
                    expected: Some(expected),
                    expected_row: Some(compare),
                }
            }
            Shape::Document => {
                let area = text_area(tr("devtools.output"));
                area.view.set_editable(false);
                Result_ {
                    frame: area.frame.clone(),
                    line: None,
                    area: Some(area),
                    expected: None,
                    expected_row: None,
                }
            }
        }
    }

    fn text(&self) -> String {
        match (&self.line, &self.area) {
            (Some(entry), _) => entry.text().to_string(),
            (_, Some(area)) => area.text(),
            _ => String::new(),
        }
    }

    fn set_text(&self, text: &str) {
        if let Some(entry) = &self.line {
            entry.set_text(text);
        }
        if let Some(area) = &self.area {
            area.set_text(text);
        }
    }

    fn verdict(&self) -> String {
        let Some(expected) = &self.expected else {
            return String::new();
        };
        let typed = expected.text().to_string();
        if typed.trim().is_empty() {
            return String::new();
        }
        if self.text().eq_ignore_ascii_case(typed.trim()) {
            tr("devtools.sum_matches")
        } else {
            tr("devtools.sum_differs")
        }
    }
}

fn caption(text: String) -> gtk::Label {
    let label = gtk::Label::builder().label(&text).xalign(0.0).build();
    label.add_css_class("dim-label");
    label
}

#[derive(Clone)]
struct TextArea {
    frame: gtk::Box,
    view: gtk::TextView,
}

impl TextArea {
    fn text(&self) -> String {
        let buffer = self.view.buffer();
        buffer.text(&buffer.start_iter(), &buffer.end_iter(), false).to_string()
    }

    fn set_text(&self, text: &str) {
        self.view.buffer().set_text(text);
    }
}

fn text_area(label: String) -> TextArea {
    let frame = gtk::Box::new(gtk::Orientation::Vertical, 4);
    let view = gtk::TextView::builder()
        .monospace(true)
        .wrap_mode(gtk::WrapMode::WordChar)
        .top_margin(6)
        .bottom_margin(6)
        .left_margin(6)
        .right_margin(6)
        .build();
    let scroll = gtk::ScrolledWindow::builder()
        .child(&view)
        .vexpand(true)
        .has_frame(true)
        .build();
    frame.append(&caption(label));
    frame.append(&scroll);
    TextArea { frame, view }
}
