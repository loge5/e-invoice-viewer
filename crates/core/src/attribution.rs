//! Attributing validation findings to fields, sections or the whole
//! document (spec 0011).

use crate::fields::{FieldModel, within};
use crate::validate::Finding;

/// Where a finding belongs; indices into the field model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Field { section: usize, field: usize },
    Section(usize),
    Document,
}

/// Attributes a finding, in this order:
///
/// 1. the field whose location equals the finding's location;
/// 2. among the fields within the finding's location, a field of the first
///    business term named in the finding that has one;
/// 3. the deepest section whose element contains the finding's location,
///    except sections of the root element;
/// 4. the document.
pub fn attribute(model: &FieldModel, finding: &Finding) -> Target {
    let location = finding.location.as_str();
    let field = model
        .fields()
        .find(|(_, _, f)| f.location == location)
        .or_else(|| {
            finding.business_terms.iter().find_map(|term| {
                model
                    .fields()
                    .find(|(_, _, f)| f.id == *term && within(&f.location, location))
            })
        });
    if let Some((section, field, _)) = field {
        return Target::Field { section, field };
    }
    model
        .section_at(location)
        .map_or(Target::Document, Target::Section)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn within_respects_step_boundaries() {
        assert!(within("/a/b[2]/c", "/a/b[2]"));
        assert!(within("/a/b", "/a/b"));
        assert!(!within("/a/b[2]", "/a/b"));
        assert!(!within("/a/bc", "/a/b"));
    }
}
