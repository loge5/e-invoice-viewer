//! XML namespaces of the invoice syntaxes.

// CII (UN/CEFACT Cross Industry Invoice).
pub(crate) const RSM: &str = "urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100";
pub(crate) const RAM: &str =
    "urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100";
pub(crate) const UDT: &str = "urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100";
pub(crate) const QDT: &str = "urn:un:unece:uncefact:data:standard:QualifiedDataType:100";

// UBL 2.1.
pub(crate) const UBL_INVOICE: &str = "urn:oasis:names:specification:ubl:schema:xsd:Invoice-2";
pub(crate) const UBL_CREDIT_NOTE: &str =
    "urn:oasis:names:specification:ubl:schema:xsd:CreditNote-2";
pub(crate) const CAC: &str =
    "urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2";
pub(crate) const CBC: &str = "urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2";
pub(crate) const EXT: &str =
    "urn:oasis:names:specification:ubl:schema:xsd:CommonExtensionComponents-2";
