//! Where the business terms live in a UBL invoice or credit note (spec 0006).
//!
//! Sections, field ids, labels and display rules are the same as for CII
//! (`cii.rs`); only the paths differ. Paths follow the UBL syntax binding of
//! EN 16931.

use super::terms::*;
use super::{
    Condition, Format, SectionDef, Step, SupportingDocumentsDef, SyntaxDef, absolute, field,
    from_attribute, repeatable,
};
use crate::ns::{CAC, CBC, EXT, UBL_CREDIT_NOTE, UBL_INVOICE};

/// Namespace prefixes used in locations, as in the EN 16931 UBL rules.
const PREFIXES: &[(&str, &str)] = &[
    ("ubl", UBL_INVOICE),
    ("cn", UBL_CREDIT_NOTE),
    ("cac", CAC),
    ("cbc", CBC),
    ("ext", EXT),
];

/// The UBL mapping.
pub const SYNTAX: SyntaxDef = SyntaxDef {
    prefixes: PREFIXES,
    currency_path: CURRENCY_PATH,
    sections: &[HEADER_SECTIONS, LINE_SECTIONS, TRAILER_SECTIONS],
    supporting_documents: SupportingDocumentsDef {
        // The same element holds the invoiced object identifier (BT-18,
        // type code 130), which is not an attachment.
        anchor: &[cac("AdditionalDocumentReference")
            .with(Condition::NotChildText(&[cbc("DocumentTypeCode")], "130"))],
        binary: &[cac("Attachment"), cbc("EmbeddedDocumentBinaryObject")],
        address: &[cac("Attachment"), cac("ExternalReference"), cbc("URI")],
        description: &[&[cbc("DocumentDescription")], &[cbc("ID")]],
    },
};

const fn cac(name: &'static str) -> Step {
    Step::new(CAC, name)
}

const fn cbc(name: &'static str) -> Step {
    Step::new(CBC, name)
}

/// Path from the root to the invoice currency (BT-5), used for amounts.
const CURRENCY_PATH: &[Step] = &[cbc("DocumentCurrencyCode")];

const TAX_SCHEME_ID: &[Step] = &[cac("TaxScheme"), cbc("ID")];
/// The party's VAT registration (BT-31, BT-48).
const VAT_SCHEME: Step = cac("PartyTaxScheme").with(Condition::ChildText(TAX_SCHEME_ID, "VAT"));
/// The party's other tax registration (BT-32).
const OTHER_TAX_SCHEME: Step =
    cac("PartyTaxScheme").with(Condition::OtherChildText(TAX_SCHEME_ID, "VAT"));
const ADDRESS: Step = cac("PostalAddress");
const LEGAL_ENTITY: Step = cac("PartyLegalEntity");
const CONTACT: Step = cac("Contact");

const HEADER_SECTIONS: &[SectionDef] = &[
    SectionDef {
        title: INVOICE,
        anchor: &[],
        repeatable: false,
        title_suffix: None,
        fields: &[
            field(BT_1, &[cbc("ID")], Format::Text),
            field(BT_2, &[cbc("IssueDate")], Format::IsoDate),
            field(
                BT_3,
                &[cbc("InvoiceTypeCode").or("CreditNoteTypeCode")],
                Format::InvoiceType,
            ),
            field(BT_5, CURRENCY_PATH, Format::Text),
            // Invoice: cbc:DueDate; credit note: in the payment means.
            field(BT_9, &[cbc("DueDate")], Format::IsoDate),
            field(
                BT_9,
                &[cac("PaymentMeans"), cbc("PaymentDueDate")],
                Format::IsoDate,
            ),
            field(BT_10, &[cbc("BuyerReference")], Format::Text),
            field(
                BT_24,
                &[cbc("CustomizationID")],
                Format::SpecificationIdentifier,
            ),
            repeatable(BT_22, &[cbc("Note")], Format::Text),
        ],
    },
    SectionDef {
        title: SELLER,
        anchor: &[cac("AccountingSupplierParty"), cac("Party")],
        repeatable: false,
        title_suffix: None,
        fields: &[
            field(
                BT_27,
                &[LEGAL_ENTITY, cbc("RegistrationName")],
                Format::Text,
            ),
            field(
                BT_29,
                &[cac("PartyIdentification"), cbc("ID")],
                Format::Text,
            ),
            field(BT_31, &[VAT_SCHEME, cbc("CompanyID")], Format::Text),
            field(BT_32, &[OTHER_TAX_SCHEME, cbc("CompanyID")], Format::Text),
            field(BT_35, &[ADDRESS, cbc("StreetName")], Format::Text),
            field(BT_36, &[ADDRESS, cbc("AdditionalStreetName")], Format::Text),
            field(BT_38, &[ADDRESS, cbc("PostalZone")], Format::Text),
            field(BT_37, &[ADDRESS, cbc("CityName")], Format::Text),
            field(
                BT_40,
                &[ADDRESS, cac("Country"), cbc("IdentificationCode")],
                Format::Text,
            ),
            field(BT_41, &[CONTACT, cbc("Name")], Format::Text),
            field(BT_42, &[CONTACT, cbc("Telephone")], Format::Text),
            field(BT_43, &[CONTACT, cbc("ElectronicMail")], Format::Text),
        ],
    },
    SectionDef {
        title: BUYER,
        anchor: &[cac("AccountingCustomerParty"), cac("Party")],
        repeatable: false,
        title_suffix: None,
        fields: &[
            field(
                BT_44,
                &[LEGAL_ENTITY, cbc("RegistrationName")],
                Format::Text,
            ),
            field(
                BT_46,
                &[cac("PartyIdentification"), cbc("ID")],
                Format::Text,
            ),
            field(BT_48, &[VAT_SCHEME, cbc("CompanyID")], Format::Text),
            field(BT_50, &[ADDRESS, cbc("StreetName")], Format::Text),
            field(BT_51, &[ADDRESS, cbc("AdditionalStreetName")], Format::Text),
            field(BT_53, &[ADDRESS, cbc("PostalZone")], Format::Text),
            field(BT_52, &[ADDRESS, cbc("CityName")], Format::Text),
            field(
                BT_55,
                &[ADDRESS, cac("Country"), cbc("IdentificationCode")],
                Format::Text,
            ),
        ],
    },
];

const ITEM: Step = cac("Item");
const QUANTITY: [Step; 1] = [cbc("InvoicedQuantity").or("CreditedQuantity")];
const LINE_TAX: [Step; 2] = [ITEM, cac("ClassifiedTaxCategory")];

/// One section per invoice (or credit note) line, BG-25.
const LINE_SECTIONS: &[SectionDef] = &[SectionDef {
    title: LINE,
    anchor: &[cac("InvoiceLine").or("CreditNoteLine")],
    repeatable: true,
    title_suffix: Some(&[cbc("ID")]),
    fields: &[
        field(BT_126, &[cbc("ID")], Format::Text),
        repeatable(BT_127, &[cbc("Note")], Format::Text),
        field(BT_153, &[ITEM, cbc("Name")], Format::Text),
        field(BT_154, &[ITEM, cbc("Description")], Format::Text),
        field(
            BT_155,
            &[ITEM, cac("SellersItemIdentification"), cbc("ID")],
            Format::Text,
        ),
        field(BT_129, &QUANTITY, Format::Quantity),
        from_attribute(BT_130, &QUANTITY, "unitCode"),
        field(BT_146, &[cac("Price"), cbc("PriceAmount")], Format::Amount),
        field(
            BT_149,
            &[cac("Price"), cbc("BaseQuantity")],
            Format::Quantity,
        ),
        field(BT_131, &[cbc("LineExtensionAmount")], Format::Amount),
        field(
            BT_151,
            &[LINE_TAX[0], LINE_TAX[1], cbc("ID")],
            Format::VatCategory,
        ),
        field(
            BT_152,
            &[LINE_TAX[0], LINE_TAX[1], cbc("Percent")],
            Format::Text,
        ),
        field(BT_128, &[cac("DocumentReference"), cbc("ID")], Format::Text),
        field(BT_133, &[cbc("AccountingCost")], Format::Text),
    ],
}];

const PAYMENT_MEANS: Step = cac("PaymentMeans");
const PAYEE_ACCOUNT: Step = cac("PayeeFinancialAccount");
const MONETARY_TOTAL: Step = cac("LegalMonetaryTotal");

const TRAILER_SECTIONS: &[SectionDef] = &[
    SectionDef {
        title: PAYMENT,
        // The payment terms of UBL are outside the payment means.
        anchor: &[],
        repeatable: false,
        title_suffix: None,
        fields: &[
            field(
                BT_81,
                &[PAYMENT_MEANS, cbc("PaymentMeansCode")],
                Format::PaymentMeans,
            ),
            field(BT_83, &[PAYMENT_MEANS, cbc("PaymentID")], Format::Text),
            field(
                BT_84,
                &[PAYMENT_MEANS, PAYEE_ACCOUNT, cbc("ID")],
                Format::Iban,
            ),
            field(
                BT_85,
                &[PAYMENT_MEANS, PAYEE_ACCOUNT, cbc("Name")],
                Format::Text,
            ),
            field(
                BT_86,
                &[
                    PAYMENT_MEANS,
                    PAYEE_ACCOUNT,
                    cac("FinancialInstitutionBranch"),
                    cbc("ID"),
                ],
                Format::Text,
            ),
            field(BT_20, &[cac("PaymentTerms"), cbc("Note")], Format::Text),
        ],
    },
    SectionDef {
        title: VAT_BREAKDOWN,
        anchor: &[cac("TaxTotal"), cac("TaxSubtotal")],
        repeatable: true,
        title_suffix: None,
        fields: &[
            field(BT_116, &[cbc("TaxableAmount")], Format::Amount),
            field(BT_117, &[cbc("TaxAmount")], Format::Amount),
            field(
                BT_118,
                &[cac("TaxCategory"), cbc("ID")],
                Format::VatCategory,
            ),
            field(BT_119, &[cac("TaxCategory"), cbc("Percent")], Format::Text),
            field(
                BT_120,
                &[cac("TaxCategory"), cbc("TaxExemptionReason")],
                Format::Text,
            ),
        ],
    },
    SectionDef {
        title: TOTALS,
        anchor: &[MONETARY_TOTAL],
        repeatable: false,
        title_suffix: None,
        fields: &[
            field(BT_106, &[cbc("LineExtensionAmount")], Format::Amount),
            field(BT_107, &[cbc("AllowanceTotalAmount")], Format::Amount),
            field(BT_108, &[cbc("ChargeTotalAmount")], Format::Amount),
            field(BT_109, &[cbc("TaxExclusiveAmount")], Format::Amount),
            // The total VAT amount is outside the monetary total, in the
            // VAT total with the breakdown; a VAT total in accounting
            // currency (BT-111) has none.
            absolute(
                BT_110,
                &[
                    cac("TaxTotal").with(Condition::HasChild(&[cac("TaxSubtotal")])),
                    cbc("TaxAmount"),
                ],
                Format::Amount,
            ),
            field(BT_112, &[cbc("TaxInclusiveAmount")], Format::Amount),
            field(BT_113, &[cbc("PrepaidAmount")], Format::Amount),
            field(BT_114, &[cbc("PayableRoundingAmount")], Format::Amount),
            field(BT_115, &[cbc("PayableAmount")], Format::Amount),
        ],
    },
];
