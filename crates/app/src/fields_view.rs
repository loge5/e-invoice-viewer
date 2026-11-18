//! The field pane: one group per section, one row per business term, with
//! the validation findings at the fields and a summary at the end
//! (specs 0007, 0011).

use std::rc::Rc;
use std::time::Duration;

use adw::prelude::*;
use e_invoice_viewer_core::attribution::{Target, attribute};
use e_invoice_viewer_core::fields::{Field, FieldModel};
use e_invoice_viewer_core::validate::{Finding, Severity};
use gtk::{gio, glib, graphene};

use crate::texts::{self, ValidationState};

/// How long a field stays highlighted after jumping to it.
const HIGHLIGHT: Duration = Duration::from_millis(2500);
/// CSS class of a highlighted field or section (see `main.rs`).
pub const HIGHLIGHT_CLASS: &str = "finding-highlight";

/// Severities in the order of the summary.
const SEVERITIES: [Severity; 3] = [Severity::Error, Severity::Warning, Severity::Information];

/// The field pane of one invoice. Cheap to clone.
#[derive(Clone)]
pub struct FieldsView {
    inner: Rc<Inner>,
}

struct Inner {
    model: FieldModel,
    scroller: gtk::ScrolledWindow,
    content: gtk::Box,
    groups: Vec<adw::PreferencesGroup>,
    /// Findings attributed to a section, shown above its fields.
    section_findings: Vec<gtk::Box>,
    rows: Vec<Vec<FieldRow>>,
    summary: adw::PreferencesGroup,
    summary_content: gtk::Box,
}

struct FieldRow {
    row: gtk::ListBoxRow,
    findings: gtk::Box,
    text: String,
}

impl FieldsView {
    /// Builds the scrollable list of all sections of the field model, then
    /// `attachments` (0013) and the validation summary.
    pub fn new(model: FieldModel, attachments: &impl IsA<gtk::Widget>) -> Self {
        let content = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(18)
            .margin_top(12)
            .margin_bottom(24)
            .margin_start(12)
            .margin_end(12)
            .build();

        let mut groups = Vec::new();
        let mut section_findings = Vec::new();
        let mut rows = Vec::new();
        for section in &model.sections {
            let group = adw::PreferencesGroup::builder()
                .title(glib::markup_escape_text(&section.title).as_str())
                .build();
            let findings = findings_box();
            group.add(&findings);
            let list = gtk::ListBox::builder()
                .selection_mode(gtk::SelectionMode::None)
                .css_classes(["boxed-list"])
                .build();
            let section_rows: Vec<FieldRow> = section.fields.iter().map(field_row).collect();
            for field_row in &section_rows {
                list.append(&field_row.row);
            }
            group.add(&list);
            content.append(&group);
            groups.push(group);
            section_findings.push(findings);
            rows.push(section_rows);
        }

        content.append(attachments);

        let summary_content = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(12)
            .build();
        let summary = adw::PreferencesGroup::builder()
            .title(texts::VALIDATION_TITLE)
            .build();
        summary.add(&summary_content);
        content.append(&summary);

        // Keeps the rows readable on wide windows instead of stretching them.
        let clamp = adw::Clamp::builder()
            .maximum_size(900)
            .tightening_threshold(600)
            .child(&content)
            .build();
        let scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .child(&clamp)
            .build();

        Self {
            inner: Rc::new(Inner {
                model,
                scroller,
                content,
                groups,
                section_findings,
                rows,
                summary,
                summary_content,
            }),
        }
    }

    pub fn widget(&self) -> gtk::Widget {
        self.inner.scroller.clone().upcast()
    }

    /// Shows a validation state: markers at fields and sections, counts at
    /// the section headers and the summary.
    pub fn set_state(&self, state: &ValidationState) {
        let inner = &self.inner;
        for (index, group) in inner.groups.iter().enumerate() {
            group.set_description(None);
            clear(&inner.section_findings[index]);
            for field_row in &inner.rows[index] {
                clear(&field_row.findings);
                set_accessible_text(&field_row.row, &field_row.text);
            }
        }
        remove_children(&inner.summary_content);

        let headline = gtk::Label::builder()
            .label(texts::status_text(state))
            .xalign(0.0)
            .wrap(true)
            .css_classes(["title-4"])
            .build();
        inner.summary_content.append(&headline);
        let ValidationState::Done(validation) = state else {
            return;
        };
        for text in [
            texts::rule_sets_text(validation),
            validation.version_note.clone(),
        ]
        .into_iter()
        .flatten()
        {
            inner.summary_content.append(&dim_label(&text));
        }

        let attributed: Vec<(&Finding, Target)> = validation
            .findings
            .iter()
            .map(|finding| (finding, attribute(&inner.model, finding)))
            .collect();

        // At the fields and sections.
        for (finding, target) in &attributed {
            match *target {
                Target::Field { section, field } => {
                    show_in(&inner.rows[section][field].findings, finding);
                }
                Target::Section(section) => show_in(&inner.section_findings[section], finding),
                Target::Document => {}
            }
        }
        for (index, group) in inner.groups.iter().enumerate() {
            let severities = attributed.iter().filter_map(|(finding, target)| {
                let in_section = match *target {
                    Target::Field { section, .. } | Target::Section(section) => section == index,
                    Target::Document => false,
                };
                in_section.then_some(finding.severity)
            });
            group.set_description(texts::section_count_text(severities).as_deref());
            for (field_index, field_row) in inner.rows[index].iter().enumerate() {
                let texts: Vec<String> = attributed
                    .iter()
                    .filter(|(_, target)| {
                        *target
                            == Target::Field {
                                section: index,
                                field: field_index,
                            }
                    })
                    .map(|(finding, _)| texts::finding_text(finding))
                    .collect();
                if !texts.is_empty() {
                    set_accessible_text(
                        &field_row.row,
                        &format!("{}; {}", field_row.text, texts.join("; ")),
                    );
                }
            }
        }

        // The summary, grouped by severity.
        for severity in SEVERITIES {
            let entries: Vec<&(&Finding, Target)> = attributed
                .iter()
                .filter(|(finding, _)| finding.severity == severity)
                .collect();
            if entries.is_empty() {
                continue;
            }
            let heading = gtk::Label::builder()
                .label(texts::severity_name(severity, entries.len()))
                .xalign(0.0)
                .css_classes(["heading"])
                .build();
            let list = gtk::ListBox::builder()
                .selection_mode(gtk::SelectionMode::None)
                .css_classes(["boxed-list"])
                .build();
            let mut targets = Vec::new();
            for (finding, target) in entries {
                list.append(&self.summary_row(finding, target));
                targets.push(*target);
            }
            let view = self.clone();
            list.connect_row_activated(move |_, row| {
                if let Some(target) = usize::try_from(row.index())
                    .ok()
                    .and_then(|index| targets.get(index))
                {
                    view.jump_to(target);
                }
            });
            inner.summary_content.append(&heading);
            inner.summary_content.append(&list);
        }
    }

    /// One summary entry: icon, rule ID with where it belongs, and message.
    fn summary_row(&self, finding: &Finding, target: &Target) -> gtk::ListBoxRow {
        let place = texts::target_text(&self.inner.model, target);
        let title = gtk::Label::builder()
            .label(format!("{} · {place}", finding.rule_id))
            .xalign(0.0)
            .wrap(true)
            .css_classes(["heading"])
            .build();
        let message = gtk::Label::builder()
            .label(&finding.message)
            .xalign(0.0)
            .wrap(true)
            .wrap_mode(gtk::pango::WrapMode::WordChar)
            .selectable(true)
            .build();
        let text = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .hexpand(true)
            .build();
        text.append(&title);
        text.append(&message);
        let line = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(12)
            .margin_end(12)
            .build();
        line.append(&severity_icon(finding.severity));
        line.append(&text);
        let jumpable = *target != Target::Document;
        let row = gtk::ListBoxRow::builder()
            .child(&line)
            .activatable(jumpable)
            .build();
        set_accessible_text(&row, &format!("{} · {place}", texts::finding_text(finding)));
        row
    }

    /// Scrolls to the summary.
    pub fn scroll_to_summary(&self) {
        let view = self.clone();
        // After switching from the XML view, wait for the layout.
        glib::idle_add_local_once(move || view.scroll_to(&view.inner.summary.clone().upcast()));
    }

    /// Scrolls to the field or section of a finding and highlights it.
    fn jump_to(&self, target: &Target) {
        let widget: gtk::Widget = match *target {
            Target::Field { section, field } => {
                self.inner.rows[section][field].row.clone().upcast()
            }
            Target::Section(section) => self.inner.groups[section].clone().upcast(),
            Target::Document => return,
        };
        self.scroll_to(&widget);
        widget.add_css_class(HIGHLIGHT_CLASS);
        glib::timeout_add_local_once(HIGHLIGHT, move || widget.remove_css_class(HIGHLIGHT_CLASS));
    }

    fn scroll_to(&self, widget: &gtk::Widget) {
        let Some(point) = widget.compute_point(&self.inner.content, &graphene::Point::zero())
        else {
            return;
        };
        let adjustment = self.inner.scroller.vadjustment();
        adjustment.set_value(f64::from(point.y()) - 12.0);
    }
}

/// A vertical box for findings, hidden while empty.
fn findings_box() -> gtk::Box {
    gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .visible(false)
        .build()
}

fn show_in(container: &gtk::Box, finding: &Finding) {
    container.append(&finding_line(finding));
    container.set_visible(true);
}

/// Empties a findings box and hides it.
fn clear(container: &gtk::Box) {
    remove_children(container);
    container.set_visible(false);
}

fn remove_children(container: &gtk::Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

/// A finding at a field or section header: icon and `BR-CO-15: message`, in
/// the colour of its severity.
fn finding_line(finding: &Finding) -> gtk::Widget {
    let label = gtk::Label::builder()
        .label(format!("{}: {}", finding.rule_id, finding.message))
        .xalign(0.0)
        .hexpand(true)
        .wrap(true)
        .wrap_mode(gtk::pango::WrapMode::WordChar)
        .selectable(true)
        .css_classes([severity_class(finding.severity)])
        .build();
    let line = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .tooltip_text(texts::finding_text(finding))
        .build();
    line.append(&severity_icon(finding.severity));
    line.append(&label);
    line.update_property(&[gtk::accessible::Property::Label(&texts::finding_text(
        finding,
    ))]);
    line.upcast()
}

fn severity_icon(severity: Severity) -> gtk::Image {
    let icon = match severity {
        Severity::Error => "dialog-error-symbolic",
        Severity::Warning => "dialog-warning-symbolic",
        Severity::Information => "dialog-information-symbolic",
    };
    let image = gtk::Image::builder()
        .icon_name(icon)
        .valign(gtk::Align::Start)
        .css_classes([severity_class(severity)])
        .build();
    image.update_property(&[gtk::accessible::Property::Label(texts::severity_name(
        severity, 1,
    ))]);
    image
}

/// libadwaita style class for the colour of a severity.
pub fn severity_class(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Information => "accent",
    }
}

fn dim_label(text: &str) -> gtk::Label {
    gtk::Label::builder()
        .label(text)
        .xalign(0.0)
        .wrap(true)
        .css_classes(["dim-label"])
        .build()
}

fn set_accessible_text(row: &gtk::ListBoxRow, text: &str) {
    row.set_tooltip_text(Some(text));
    row.update_property(&[gtk::accessible::Property::Label(text)]);
}

/// One row: business term number, label and the selectable display value,
/// with "Wert kopieren" in the value's context menu, and its findings below.
fn field_row(field: &Field) -> FieldRow {
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
        .build();
    line.append(&id);
    line.append(&label);
    line.append(&value);

    let findings = findings_box();
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(6)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(12)
        .margin_end(12)
        .build();
    content.append(&line);
    content.append(&findings);

    let text = texts::field_row_text(field);
    let row = gtk::ListBoxRow::builder()
        .child(&content)
        .activatable(false)
        .build();
    set_accessible_text(&row, &text);

    let actions = gio::SimpleActionGroup::new();
    let copy = gio::SimpleAction::new("copy-value", None);
    let raw = texts::copy_value(field).to_string();
    let target = row.clone();
    copy.connect_activate(move |_, _| target.clipboard().set_text(&raw));
    actions.add_action(&copy);
    row.insert_action_group("row", Some(&actions));
    FieldRow {
        row,
        findings,
        text,
    }
}
