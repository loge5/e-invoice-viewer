//! Looking up sections and fields of a field model.

use e_invoice_viewer_core::fields::{Field, FieldModel, Section, map_invoice};

pub fn model(xml: &str) -> FieldModel {
    map_invoice(xml.as_bytes()).expect("field model")
}

/// All sections with this title, in order.
pub fn sections<'a>(model: &'a FieldModel, title: &str) -> Vec<&'a Section> {
    model
        .sections
        .iter()
        .filter(|section| section.title == title)
        .collect()
}

/// The only section with this title.
pub fn section<'a>(model: &'a FieldModel, title: &str) -> &'a Section {
    match sections(model, title)[..] {
        [section] => section,
        [] => panic!("no section {title:?}"),
        _ => panic!("section {title:?} is not unique"),
    }
}

/// The first field with this business term in the section.
pub fn field<'a>(section: &'a Section, id: &str) -> &'a Field {
    section
        .fields
        .iter()
        .find(|field| field.id == id)
        .unwrap_or_else(|| panic!("no field {id} in {:?}", section.title))
}
