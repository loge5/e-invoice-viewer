//! The section "Anhänge" (spec 0013).

use adw::prelude::*;
use e_invoice_viewer_core::attachments::{Attachment, Content};

use crate::texts;

/// The section listing `attachments`; `on_save` is called with the
/// attachment whose save button was clicked and its 1-based position.
pub fn group<'a>(
    attachments: impl IntoIterator<Item = &'a Attachment>,
    on_save: impl Fn(&Attachment, usize) + Clone + 'static,
) -> adw::PreferencesGroup {
    let attachments: Vec<&Attachment> = attachments.into_iter().collect();
    let group = adw::PreferencesGroup::builder()
        .title(texts::t(texts::ATTACHMENTS_TITLE))
        .build();
    if attachments.is_empty() {
        let label = gtk::Label::builder()
            .label(texts::t(texts::NO_ATTACHMENTS))
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
    for (index, attachment) in attachments.into_iter().enumerate() {
        let position = index + 1;
        // File names and descriptions come from the invoice: never markup.
        let row = adw::ActionRow::builder()
            .use_markup(false)
            .title(attachment.display_name(position))
            .subtitle(texts::attachment_details(attachment))
            .subtitle_selectable(true)
            .build();
        if matches!(attachment.content, Content::Embedded(_)) {
            let button = gtk::Button::builder()
                .icon_name("document-save-symbolic")
                .tooltip_text(texts::t(texts::SAVE_ATTACHMENT))
                .valign(gtk::Align::Center)
                .css_classes(["flat"])
                .build();
            button.update_property(&[gtk::accessible::Property::Label(texts::t(
                texts::SAVE_ATTACHMENT,
            ))]);
            let (on_save, attachment) = (on_save.clone(), attachment.clone());
            button.connect_clicked(move |_| on_save(&attachment, position));
            row.add_suffix(&button);
        }
        list.append(&row);
    }
    group.add(&list);
    group
}
