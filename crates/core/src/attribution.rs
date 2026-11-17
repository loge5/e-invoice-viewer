//! Attributing validation findings to fields, sections or the whole
//! document (spec 0011).

use crate::fields::FieldModel;
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
    let fields = || {
        model.sections.iter().enumerate().flat_map(|(section, s)| {
            s.fields
                .iter()
                .enumerate()
                .map(move |(field, f)| (section, field, f))
        })
    };

    if let Some((section, field, _)) = fields().find(|(_, _, f)| f.location == location) {
        return Target::Field { section, field };
    }
    for term in &finding.business_terms {
        if let Some((section, field, _)) =
            fields().find(|(_, _, f)| f.id == *term && within(&f.location, location))
        {
            return Target::Field { section, field };
        }
    }
    model
        .sections
        .iter()
        .enumerate()
        .filter(|(_, section)| {
            is_below_root(&section.location) && within(location, &section.location)
        })
        .max_by_key(|(_, section)| section.location.len())
        .map_or(Target::Document, |(section, _)| Target::Section(section))
}

/// Whether `location` is `ancestor` or lies below it.
fn within(location: &str, ancestor: &str) -> bool {
    location
        .strip_prefix(ancestor)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
}

/// Whether a location has more than one step, i.e. is not the root element.
fn is_below_root(location: &str) -> bool {
    location.trim_start_matches('/').contains('/')
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
