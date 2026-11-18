//! The section "Anhänge" (spec 0013).

use adw::prelude::*;
use e_invoice_viewer_core::attachments::{Attachment, Content};

use crate::texts;

/// The section listing `attachments`; `on_save` is called with the index of
/// the attachment whose save button was clicked.
pub fn group(
    attachments: &[Attachment],
    on_save: impl Fn(usize) + Clone + 'static,
) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title(texts::ATTACHMENTS_TITLE)
        .build();
    if attachments.is_empty() {
        let label = gtk::Label::builder()
            .label(texts::NO_ATTACHMENTS)
            .xalign(0.0)
            .css_classes(["dim-label"])
            .build();
        group.add(&label);
        return group;
    }
    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();
    for (index, attachment) in attachments.iter().enumerate() {
        // File names and descriptions come from the invoice: never markup.
        let row = adw::ActionRow::builder()
            .use_markup(false)
            .title(attachment.display_name(index + 1))
            .subtitle(texts::attachment_details(attachment))
            .subtitle_selectable(true)
            .build();
        if matches!(attachment.content, Content::Embedded(_)) {
            let button = gtk::Button::builder()
                .icon_name("document-save-symbolic")
                .tooltip_text(texts::SAVE_ATTACHMENT)
                .valign(gtk::Align::Center)
                .css_classes(["flat"])
                .build();
            button.update_property(&[gtk::accessible::Property::Label(texts::SAVE_ATTACHMENT)]);
            let on_save = on_save.clone();
            button.connect_clicked(move |_| on_save(index));
            row.add_suffix(&button);
        }
        list.append(&row);
    }
    group.add(&list);
    group
}
