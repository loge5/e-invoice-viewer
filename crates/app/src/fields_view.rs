//! The field pane: one group per section, one row per business term.

use adw::prelude::*;
use e_invoice_viewer_core::fields::{Field, FieldModel};
use gtk::{gio, glib};

use crate::texts;

/// Builds the scrollable list of all sections of the field model.
pub fn build(model: &FieldModel) -> gtk::Widget {
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(18)
        .margin_top(12)
        .margin_bottom(24)
        .margin_start(12)
        .margin_end(12)
        .build();

    for section in &model.sections {
        let group = adw::PreferencesGroup::builder()
            .title(glib::markup_escape_text(&section.title).as_str())
            .build();
        let list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(["boxed-list"])
            .build();
        for field in &section.fields {
            list.append(&row(field));
        }
        group.add(&list);
        content.append(&group);
    }

    // Keeps the rows readable on wide windows instead of stretching them.
    let clamp = adw::Clamp::builder()
        .maximum_size(900)
        .tightening_threshold(600)
        .child(&content)
        .build();
    gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .child(&clamp)
        .build()
        .upcast()
}

/// One row: business term number, label and the selectable display value,
/// with "Wert kopieren" in the value's context menu.
fn row(field: &Field) -> gtk::ListBoxRow {
    let id = gtk::Label::builder()
        .label(&field.id)
        .xalign(0.0)
        .width_chars(7)
        .css_classes(["dim-label", "caption"])
        .build();
    let label = gtk::Label::builder()
        .label(&field.label)
        .xalign(0.0)
        .width_chars(18)
        .max_width_chars(28)
        .wrap(true)
        .build();
    let value = gtk::Label::builder()
        .label(&field.display)
        .xalign(0.0)
        .hexpand(true)
        .wrap(true)
        .wrap_mode(gtk::pango::WrapMode::WordChar)
        .selectable(true)
        .build();

    let menu = gio::Menu::new();
    menu.append(Some(texts::COPY_VALUE), Some("row.copy-value"));
    value.set_extra_menu(Some(&menu));

    let line = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(12)
        .margin_end(12)
        .build();
    line.append(&id);
    line.append(&label);
    line.append(&value);

    let row = gtk::ListBoxRow::builder()
        .child(&line)
        .activatable(false)
        .tooltip_text(texts::field_row_text(field))
        .build();
    row.update_property(&[gtk::accessible::Property::Label(&texts::field_row_text(
        field,
    ))]);

    let actions = gio::SimpleActionGroup::new();
    let copy = gio::SimpleAction::new("copy-value", None);
    let raw = texts::copy_value(field).to_string();
    let target = row.clone();
    copy.connect_activate(move |_, _| target.clipboard().set_text(&raw));
    actions.add_action(&copy);
    row.insert_action_group("row", Some(&actions));
    row
}
