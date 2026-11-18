//! The status line above the fields: the validation state as icon and text
//! (spec 0011); a click leads to the summary.

use adw::prelude::*;
use e_invoice_viewer_core::validate::Severity;

use crate::fields_view::severity_class;
use crate::texts::{self, ValidationState};

/// Style classes the icon may carry.
const CLASSES: [&str; 3] = ["error", "warning", "success"];

pub struct StatusLine {
    button: gtk::Button,
    icon: gtk::Image,
    label: gtk::Label,
}

impl StatusLine {
    /// A hidden status line.
    pub fn new() -> Self {
        let icon = gtk::Image::new();
        let label = gtk::Label::builder()
            .xalign(0.0)
            .wrap(true)
            .css_classes(["heading"])
            .build();
        let line = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .build();
        line.append(&icon);
        line.append(&label);
        let button = gtk::Button::builder()
            .child(&line)
            .css_classes(["flat"])
            .margin_start(6)
            .margin_end(6)
            .margin_top(6)
            .visible(false)
            .build();
        Self {
            button,
            icon,
            label,
        }
    }

    pub fn widget(&self) -> &gtk::Button {
        &self.button
    }

    pub fn connect_clicked(&self, clicked: impl Fn() + 'static) {
        self.button.connect_clicked(move |_| clicked());
    }

    /// Shows `state`.
    pub fn show(&self, state: &ValidationState) {
        let text = texts::status_text(state);
        self.label.set_label(&text);
        let (icon, class) = look(state);
        self.icon.set_icon_name(Some(icon));
        for name in CLASSES {
            self.icon.remove_css_class(name);
        }
        if let Some(class) = class {
            self.icon.add_css_class(class);
        }
        self.button
            .update_property(&[gtk::accessible::Property::Label(&text)]);
        self.button.set_tooltip_text(Some(&text));
        self.button.set_visible(true);
    }

    pub fn hide(&self) {
        self.button.set_visible(false);
    }
}

/// Icon name and style class of a validation state.
fn look(state: &ValidationState) -> (&'static str, Option<&'static str>) {
    match state {
        ValidationState::Running => ("content-loading-symbolic", None),
        ValidationState::Failed(_) => ("dialog-error-symbolic", Some("error")),
        ValidationState::Done(validation) => match validation.worst_severity() {
            Some(Severity::Error) => (
                "dialog-error-symbolic",
                Some(severity_class(Severity::Error)),
            ),
            Some(Severity::Warning) => (
                "dialog-warning-symbolic",
                Some(severity_class(Severity::Warning)),
            ),
            Some(Severity::Information) => ("dialog-information-symbolic", None),
            None if validation.not_applicable_reason.is_some() => {
                ("dialog-information-symbolic", None)
            }
            None => ("object-select-symbolic", Some("success")),
        },
    }
}

#[cfg(test)]
mod tests {
    use e_invoice_viewer_core::validate::{Finding, Validation};

    use super::*;

    fn done(severities: &[Severity], not_applicable: bool) -> ValidationState {
        ValidationState::Done(Validation {
            applied_rule_sets: Vec::new(),
            not_applicable_reason: not_applicable.then(|| "kein Profil".to_string()),
            version_note: None,
            findings: severities
                .iter()
                .map(|severity| Finding {
                    rule_id: String::new(),
                    severity: *severity,
                    message: String::new(),
                    location: String::new(),
                    business_terms: Vec::new(),
                })
                .collect(),
        })
    }

    #[test]
    fn look_follows_the_worst_finding() {
        assert_eq!(
            look(&done(&[], false)),
            ("object-select-symbolic", Some("success"))
        );
        assert_eq!(
            look(&done(&[], true)),
            ("dialog-information-symbolic", None)
        );
        assert_eq!(
            look(&done(&[Severity::Information, Severity::Warning], false)),
            ("dialog-warning-symbolic", Some("warning"))
        );
        assert_eq!(
            look(&done(&[Severity::Error], false)),
            ("dialog-error-symbolic", Some("error"))
        );
        assert_eq!(look(&ValidationState::Running).1, None);
        assert_eq!(
            look(&ValidationState::Failed(String::new())),
            ("dialog-error-symbolic", Some("error"))
        );
    }
}
