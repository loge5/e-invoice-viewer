//! XML schema check of an invoice (spec 0023, ADR 0006) with libxml2.
//!
//! The schemas are embedded in the program. libxml2 reads them through input
//! callbacks registered for an internal address scheme (`eiv-schema:/`), so
//! that nothing is read from disk or from the network. This is the only
//! module of `crates/core` with `unsafe` code: the calls into libxml2.
#![allow(unsafe_code)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::sync::Once;

use libxml::bindings as ffi;

/// The internal address scheme of the embedded schemas.
const SCHEME: &str = "eiv-schema:/";

macro_rules! embedded {
    ($($path:literal),* $(,)?) => {
        &[$(($path, include_str!(concat!("../../rules/xsd-2026-08-31/", $path)))),*]
    };
}

/// The embedded schema files, by their path below the scheme.
const FILES: &[(&str, &str)] = embedded![
    "cii/CrossIndustryInvoice_100pD16B.xsd",
    "cii/CrossIndustryInvoice_QualifiedDataType_100pD16B.xsd",
    "cii/CrossIndustryInvoice_ReusableAggregateBusinessInformationEntity_100pD16B.xsd",
    "cii/CrossIndustryInvoice_UnqualifiedDataType_100pD16B.xsd",
    "ubl/maindoc/UBL-Invoice-2.1.xsd",
    "ubl/maindoc/UBL-CreditNote-2.1.xsd",
    "ubl/common/CCTS_CCT_SchemaModule-2.1.xsd",
    "ubl/common/UBL-CommonAggregateComponents-2.1.xsd",
    "ubl/common/UBL-CommonBasicComponents-2.1.xsd",
    "ubl/common/UBL-CommonExtensionComponents-2.1.xsd",
    "ubl/common/UBL-CommonSignatureComponents-2.1.xsd",
    "ubl/common/UBL-ExtensionContentDataType-2.1.xsd",
    "ubl/common/UBL-QualifiedDataTypes-2.1.xsd",
    "ubl/common/UBL-SignatureAggregateComponents-2.1.xsd",
    "ubl/common/UBL-SignatureBasicComponents-2.1.xsd",
    "ubl/common/UBL-UnqualifiedDataTypes-2.1.xsd",
    "ubl/common/UBL-XAdESv132-2.1.xsd",
    "ubl/common/UBL-XAdESv141-2.1.xsd",
    "ubl/common/UBL-xmldsig-core-schema-2.1.xsd",
];

/// Which schema an invoice is checked against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Schema {
    Cii,
    UblInvoice,
    UblCreditNote,
}

impl Schema {
    fn main_file(self) -> &'static str {
        match self {
            Self::Cii => "cii/CrossIndustryInvoice_100pD16B.xsd",
            Self::UblInvoice => "ubl/maindoc/UBL-Invoice-2.1.xsd",
            Self::UblCreditNote => "ubl/maindoc/UBL-CreditNote-2.1.xsd",
        }
    }

    /// The schema's name as shown in the summary, e.g. `CII D16B`.
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Cii => "CII D16B",
            Self::UblInvoice | Self::UblCreditNote => "UBL 2.1",
        }
    }
}

/// One schema error: libxml2's message, and the element it concerns as the
/// positions of the element among its parent's element children, from the
/// root element down (the root element is `[0]`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SchemaError {
    pub(super) message: String,
    pub(super) element: Option<Vec<usize>>,
}

/// Checks `text` (the decoded invoice XML) against `schema`.
///
/// # Panics
///
/// If an embedded schema cannot be compiled, which the tests rule out.
pub(super) fn check(text: &str, schema: Schema) -> Vec<SchemaError> {
    init();
    let compiled = compiled(schema);
    let Ok(length) = c_int::try_from(text.len()) else {
        return vec![SchemaError {
            message: "The invoice is too large for the XML schema check".to_string(),
            element: None,
        }];
    };
    let utf8 = c"UTF-8";
    let mut errors: Vec<SchemaError> = Vec::new();
    // SAFETY: the buffer and its length come from `text`; the encoding
    // overrides a declaration in the text, which `decode` already applied.
    // The document is freed below; the error callback only runs during
    // `xmlSchemaValidateDoc`, while `errors` and the document are alive.
    unsafe {
        let doc = ffi::xmlReadMemory(
            text.as_ptr().cast::<c_char>(),
            length,
            std::ptr::null(),
            utf8.as_ptr(),
            ffi::xmlParserOption_XML_PARSE_NONET as c_int,
        );
        if doc.is_null() {
            // The invoice was parsed before (detection); nothing to report.
            return errors;
        }
        let context = ffi::xmlSchemaNewValidCtxt(compiled);
        ffi::xmlSchemaSetValidStructuredErrors(
            context,
            error_callback(),
            (&raw mut errors).cast::<c_void>(),
        );
        ffi::xmlSchemaValidateDoc(context, doc);
        ffi::xmlSchemaFreeValidCtxt(context);
        ffi::xmlFreeDoc(doc);
    }
    errors
}

/// Registers the input callbacks for the embedded schemas, once per process.
fn init() {
    static INIT: Once = Once::new();
    // SAFETY: called once, before any other use of libxml2 by this module;
    // the callbacks are plain functions that live for the whole process.
    INIT.call_once(|| unsafe {
        ffi::xmlInitParser();
        ffi::xmlRegisterInputCallbacks(
            Some(input_matches),
            Some(input_open),
            Some(input_read),
            Some(input_close),
        );
    });
}

thread_local! {
    /// Compiled schemas of this thread; never freed (process lifetime).
    static COMPILED: RefCell<HashMap<Schema, usize>> = RefCell::new(HashMap::new());
}

/// The compiled schema, compiled on first use in this thread.
fn compiled(schema: Schema) -> *mut ffi::_xmlSchema {
    let address = COMPILED.with(|cache| {
        *cache
            .borrow_mut()
            .entry(schema)
            .or_insert_with(|| compile(schema) as usize)
    });
    address as *mut ffi::_xmlSchema
}

fn compile(schema: Schema) -> *mut ffi::_xmlSchema {
    let url = CString::new(format!("{SCHEME}{}", schema.main_file())).expect("no NUL");
    let mut errors: Vec<SchemaError> = Vec::new();
    // SAFETY: the parser context reads the schema through the registered
    // callbacks; `errors` outlives the parse; the context is freed after it.
    let compiled = unsafe {
        let parser = ffi::xmlSchemaNewParserCtxt(url.as_ptr());
        ffi::xmlSchemaSetParserStructuredErrors(
            parser,
            error_callback(),
            (&raw mut errors).cast::<c_void>(),
        );
        let compiled = ffi::xmlSchemaParse(parser);
        ffi::xmlSchemaFreeParserCtxt(parser);
        compiled
    };
    assert!(
        !compiled.is_null(),
        "embedded schema {} does not compile: {errors:?}",
        schema.main_file()
    );
    compiled
}

/// The structured error callback, typed as the libxml2 version in use
/// expects it (the error pointer is `const` since libxml2 2.12; with a newer
/// libxml2 the transmute changes nothing).
#[allow(clippy::useless_transmute)]
fn error_callback() -> ffi::xmlStructuredErrorFunc {
    let callback: unsafe extern "C" fn(*mut c_void, *const ffi::_xmlError) = on_error;
    // SAFETY: only the constness of the pointer argument may differ between
    // libxml2 versions; the ABI is the same.
    Some(unsafe { std::mem::transmute(callback) })
}

/// Collects one error into the `Vec<SchemaError>` passed as user data.
unsafe extern "C" fn on_error(data: *mut c_void, error: *const ffi::_xmlError) {
    if data.is_null() || error.is_null() {
        return;
    }
    // SAFETY: `data` is the `Vec` passed at registration; `error` is valid
    // during the callback, and so is the node it points to.
    unsafe {
        let errors = &mut *data.cast::<Vec<SchemaError>>();
        let error = &*error;
        let message = if error.message.is_null() {
            String::new()
        } else {
            CStr::from_ptr(error.message).to_string_lossy().into_owned()
        };
        errors.push(SchemaError {
            message,
            element: element_positions(error.node.cast::<ffi::_xmlNode>()),
        });
    }
}

const ELEMENT_NODE: ffi::xmlElementType = ffi::xmlElementType_XML_ELEMENT_NODE;
const ATTRIBUTE_NODE: ffi::xmlElementType = ffi::xmlElementType_XML_ATTRIBUTE_NODE;

/// The positions of `node` (or, for an attribute, its element) among the
/// element children of each ancestor, from the root element down.
///
/// # Safety
///
/// `node` is null or a valid node of a document that is alive.
unsafe fn element_positions(mut node: *mut ffi::_xmlNode) -> Option<Vec<usize>> {
    // SAFETY: the caller guarantees a valid tree; links are followed only
    // while they are not null.
    unsafe {
        if !node.is_null() && (*node).type_ == ATTRIBUTE_NODE {
            node = (*node).parent;
        }
        if node.is_null() || (*node).type_ != ELEMENT_NODE {
            return None;
        }
        let mut positions = Vec::new();
        while !node.is_null() && (*node).type_ == ELEMENT_NODE {
            let mut position = 0;
            let mut sibling = (*node).prev;
            while !sibling.is_null() {
                if (*sibling).type_ == ELEMENT_NODE {
                    position += 1;
                }
                sibling = (*sibling).prev;
            }
            positions.push(position);
            node = (*node).parent;
        }
        positions.reverse();
        Some(positions)
    }
}

/// The embedded file for an address of the internal scheme.
fn embedded_file(address: &str) -> Option<&'static str> {
    let path = address.strip_prefix(SCHEME)?;
    FILES
        .iter()
        .find(|(file, _)| *file == path)
        .map(|(_, text)| *text)
}

/// An embedded file being read.
struct Reader {
    data: &'static [u8],
    position: usize,
}

unsafe extern "C" fn input_matches(address: *const c_char) -> c_int {
    if address.is_null() {
        return 0;
    }
    // SAFETY: libxml2 passes a NUL-terminated address.
    let address = unsafe { CStr::from_ptr(address) };
    c_int::from(address.to_bytes().starts_with(SCHEME.as_bytes()))
}

unsafe extern "C" fn input_open(address: *const c_char) -> *mut c_void {
    if address.is_null() {
        return std::ptr::null_mut();
    }
    // SAFETY: libxml2 passes a NUL-terminated address.
    let address = unsafe { CStr::from_ptr(address) }.to_string_lossy();
    match embedded_file(&address) {
        Some(text) => Box::into_raw(Box::new(Reader {
            data: text.as_bytes(),
            position: 0,
        }))
        .cast::<c_void>(),
        // An unknown address of the scheme fails instead of falling back to
        // a file or the network.
        None => std::ptr::null_mut(),
    }
}

unsafe extern "C" fn input_read(context: *mut c_void, buffer: *mut c_char, length: c_int) -> c_int {
    if context.is_null() || buffer.is_null() || length < 0 {
        return -1;
    }
    // SAFETY: `context` is a `Reader` from `input_open`; `buffer` has room
    // for `length` bytes.
    unsafe {
        let reader = &mut *context.cast::<Reader>();
        let rest = &reader.data[reader.position..];
        let count = rest.len().min(length as usize);
        std::ptr::copy_nonoverlapping(rest.as_ptr(), buffer.cast::<u8>(), count);
        reader.position += count;
        count as c_int
    }
}

unsafe extern "C" fn input_close(context: *mut c_void) -> c_int {
    if !context.is_null() {
        // SAFETY: `context` is a `Reader` from `input_open`, closed once.
        drop(unsafe { Box::from_raw(context.cast::<Reader>()) });
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_schema_compiles() {
        for schema in [Schema::Cii, Schema::UblInvoice, Schema::UblCreditNote] {
            assert!(!compiled(schema).is_null());
        }
    }

    #[test]
    fn schemas_are_read_from_memory_only() {
        init();
        assert!(embedded_file("eiv-schema:/cii/CrossIndustryInvoice_100pD16B.xsd").is_some());
        assert!(embedded_file("eiv-schema:/../../etc/passwd").is_none());
        // A schema that imports an address of the scheme that is not embedded
        // does not compile, and nothing else is tried.
        let xsd = format!(
            r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
                 <xs:import namespace="urn:x" schemaLocation="{SCHEME}missing.xsd"/>
                 <xs:element name="a" type="xs:string"/>
               </xs:schema>"#
        );
        let mut errors: Vec<SchemaError> = Vec::new();
        // SAFETY: as in `compile`, with an in-memory schema.
        let compiled = unsafe {
            let parser = ffi::xmlSchemaNewMemParserCtxt(
                xsd.as_ptr().cast::<c_char>(),
                c_int::try_from(xsd.len()).unwrap(),
            );
            ffi::xmlSchemaSetParserStructuredErrors(
                parser,
                error_callback(),
                (&raw mut errors).cast::<c_void>(),
            );
            let compiled = ffi::xmlSchemaParse(parser);
            ffi::xmlSchemaFreeParserCtxt(parser);
            compiled
        };
        // libxml2 treats a failed import as a warning and goes on without
        // it; either way, the missing file was not found anywhere.
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("missing.xsd")),
            "{errors:?}"
        );
        if !compiled.is_null() {
            // SAFETY: freeing the schema compiled above.
            unsafe { ffi::xmlSchemaFree(compiled) };
        }
    }

    #[test]
    fn wrong_root_is_reported_at_the_root() {
        let errors = check("<a><b/></a>", Schema::Cii);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert_eq!(errors[0].element, Some(vec![0]));
    }
}
