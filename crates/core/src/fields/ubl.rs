//! Where the business terms live in a UBL invoice or credit note (spec 0006).
//!
//! Sections, field ids, labels and display rules are the same as for CII
//! (`cii.rs`); only the paths differ. Paths follow the UBL syntax binding of
//! EN 16931.

use super::{Condition, FieldDef, Format, SectionDef, Step, SyntaxDef};

const UBL: &str = "urn:oasis:names:specification:ubl:schema:xsd:Invoice-2";
const CN: &str = "urn:oasis:names:specification:ubl:schema:xsd:CreditNote-2";
const CAC: &str = "urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2";
const CBC: &str = "urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2";

/// Namespace prefixes used in locations, as in the EN 16931 UBL rules.
const PREFIXES: &[(&str, &str)] = &[
    ("ubl", UBL),
    ("cn", CN),
    ("cac", CAC),
    ("cbc", CBC),
    (
        "ext",
        "urn:oasis:names:specification:ubl:schema:xsd:CommonExtensionComponents-2",
    ),
];

/// The UBL mapping.
pub const SYNTAX: SyntaxDef = SyntaxDef {
    prefixes: PREFIXES,
    currency_path: CURRENCY_PATH,
    sections: &[HEADER_SECTIONS, LINE_SECTIONS, TRAILER_SECTIONS],
};

const fn cac(name: &'static str) -> Step {
    Step::new(CAC, name)
}

const fn cbc(name: &'static str) -> Step {
    Step::new(CBC, name)
}

const fn field(
    id: &'static str,
    label: &'static str,
    path: &'static [Step],
    format: Format,
) -> FieldDef {
    FieldDef {
        id,
        label,
        path,
        format,
        repeatable: false,
        attribute: None,
        absolute: false,
    }
}

const fn repeatable(
    id: &'static str,
    label: &'static str,
    path: &'static [Step],
    format: Format,
) -> FieldDef {
    FieldDef {
        repeatable: true,
        ..field(id, label, path, format)
    }
}

/// A field whose path starts at the root element, not at the section anchor.
const fn absolute(
    id: &'static str,
    label: &'static str,
    path: &'static [Step],
    format: Format,
) -> FieldDef {
    FieldDef {
        absolute: true,
        ..field(id, label, path, format)
    }
}

/// A field whose value is the attribute `attribute` of the element at `path`.
const fn from_attribute(
    id: &'static str,
    label: &'static str,
    path: &'static [Step],
    attribute: &'static str,
) -> FieldDef {
    FieldDef {
        attribute: Some(attribute),
        ..field(id, label, path, Format::Text)
    }
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
        title: "Rechnung",
        anchor: &[],
        repeatable: false,
        title_suffix: None,
        fields: &[
            field("BT-1", "Rechnungsnummer", &[cbc("ID")], Format::Text),
            field(
                "BT-2",
                "Rechnungsdatum",
                &[cbc("IssueDate")],
                Format::IsoDate,
            ),
            field(
                "BT-3",
                "Rechnungsart",
                &[cbc("InvoiceTypeCode").or("CreditNoteTypeCode")],
                Format::InvoiceType,
            ),
            field("BT-5", "Währung", CURRENCY_PATH, Format::Text),
            // Invoice: cbc:DueDate; credit note: in the payment means.
            field(
                "BT-9",
                "Fälligkeitsdatum",
                &[cbc("DueDate")],
                Format::IsoDate,
            ),
            field(
                "BT-9",
                "Fälligkeitsdatum",
                &[cac("PaymentMeans"), cbc("PaymentDueDate")],
                Format::IsoDate,
            ),
            field(
                "BT-10",
                "Käuferreferenz",
                &[cbc("BuyerReference")],
                Format::Text,
            ),
            field(
                "BT-24",
                "Spezifikationskennung",
                &[cbc("CustomizationID")],
                Format::SpecificationIdentifier,
            ),
            repeatable("BT-22", "Bemerkung", &[cbc("Note")], Format::Text),
        ],
    },
    SectionDef {
        title: "Verkäufer (BG-4)",
        anchor: &[cac("AccountingSupplierParty"), cac("Party")],
        repeatable: false,
        title_suffix: None,
        fields: &[
            field(
                "BT-27",
                "Name",
                &[LEGAL_ENTITY, cbc("RegistrationName")],
                Format::Text,
            ),
            field(
                "BT-29",
                "Kennung",
                &[cac("PartyIdentification"), cbc("ID")],
                Format::Text,
            ),
            field(
                "BT-31",
                "Umsatzsteuer-ID",
                &[VAT_SCHEME, cbc("CompanyID")],
                Format::Text,
            ),
            field(
                "BT-32",
                "Steuernummer",
                &[OTHER_TAX_SCHEME, cbc("CompanyID")],
                Format::Text,
            ),
            field(
                "BT-35",
                "Straße",
                &[ADDRESS, cbc("StreetName")],
                Format::Text,
            ),
            field(
                "BT-36",
                "Adresszusatz",
                &[ADDRESS, cbc("AdditionalStreetName")],
                Format::Text,
            ),
            field("BT-38", "PLZ", &[ADDRESS, cbc("PostalZone")], Format::Text),
            field("BT-37", "Ort", &[ADDRESS, cbc("CityName")], Format::Text),
            field(
                "BT-40",
                "Land",
                &[ADDRESS, cac("Country"), cbc("IdentificationCode")],
                Format::Text,
            ),
            field(
                "BT-41",
                "Kontaktperson",
                &[CONTACT, cbc("Name")],
                Format::Text,
            ),
            field(
                "BT-42",
                "Telefon",
                &[CONTACT, cbc("Telephone")],
                Format::Text,
            ),
            field(
                "BT-43",
                "E-Mail",
                &[CONTACT, cbc("ElectronicMail")],
                Format::Text,
            ),
        ],
    },
    SectionDef {
        title: "Käufer (BG-7)",
        anchor: &[cac("AccountingCustomerParty"), cac("Party")],
        repeatable: false,
        title_suffix: None,
        fields: &[
            field(
                "BT-44",
                "Name",
                &[LEGAL_ENTITY, cbc("RegistrationName")],
                Format::Text,
            ),
            field(
                "BT-46",
                "Kennung",
                &[cac("PartyIdentification"), cbc("ID")],
                Format::Text,
            ),
            field(
                "BT-48",
                "Umsatzsteuer-ID",
                &[VAT_SCHEME, cbc("CompanyID")],
                Format::Text,
            ),
            field(
                "BT-50",
                "Straße",
                &[ADDRESS, cbc("StreetName")],
                Format::Text,
            ),
            field(
                "BT-51",
                "Adresszusatz",
                &[ADDRESS, cbc("AdditionalStreetName")],
                Format::Text,
            ),
            field("BT-53", "PLZ", &[ADDRESS, cbc("PostalZone")], Format::Text),
            field("BT-52", "Ort", &[ADDRESS, cbc("CityName")], Format::Text),
            field(
                "BT-55",
                "Land",
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
    title: "Position",
    anchor: &[cac("InvoiceLine").or("CreditNoteLine")],
    repeatable: true,
    title_suffix: Some(&[cbc("ID")]),
    fields: &[
        field("BT-126", "Positionsnummer", &[cbc("ID")], Format::Text),
        repeatable("BT-127", "Positionsbemerkung", &[cbc("Note")], Format::Text),
        field("BT-153", "Artikelname", &[ITEM, cbc("Name")], Format::Text),
        field(
            "BT-154",
            "Artikelbeschreibung",
            &[ITEM, cbc("Description")],
            Format::Text,
        ),
        field(
            "BT-155",
            "Artikelnummer",
            &[ITEM, cac("SellersItemIdentification"), cbc("ID")],
            Format::Text,
        ),
        field("BT-129", "Menge", &QUANTITY, Format::Quantity),
        from_attribute("BT-130", "Einheit", &QUANTITY, "unitCode"),
        field(
            "BT-146",
            "Nettopreis",
            &[cac("Price"), cbc("PriceAmount")],
            Format::Amount,
        ),
        field(
            "BT-149",
            "Basismenge",
            &[cac("Price"), cbc("BaseQuantity")],
            Format::Quantity,
        ),
        field(
            "BT-131",
            "Nettobetrag",
            &[cbc("LineExtensionAmount")],
            Format::Amount,
        ),
        field(
            "BT-151",
            "Steuerkategorie",
            &[LINE_TAX[0], LINE_TAX[1], cbc("ID")],
            Format::VatCategory,
        ),
        field(
            "BT-152",
            "Steuersatz",
            &[LINE_TAX[0], LINE_TAX[1], cbc("Percent")],
            Format::Text,
        ),
        field(
            "BT-128",
            "Objektkennung",
            &[cac("DocumentReference"), cbc("ID")],
            Format::Text,
        ),
        field(
            "BT-133",
            "Buchungsreferenz des Käufers",
            &[cbc("AccountingCost")],
            Format::Text,
        ),
    ],
}];

const PAYMENT_MEANS: Step = cac("PaymentMeans");
const PAYEE_ACCOUNT: Step = cac("PayeeFinancialAccount");
const MONETARY_TOTAL: Step = cac("LegalMonetaryTotal");

const TRAILER_SECTIONS: &[SectionDef] = &[
    SectionDef {
        title: "Zahlung (BG-16)",
        // The payment terms of UBL are outside the payment means.
        anchor: &[],
        repeatable: false,
        title_suffix: None,
        fields: &[
            field(
                "BT-81",
                "Zahlungsart",
                &[PAYMENT_MEANS, cbc("PaymentMeansCode")],
                Format::PaymentMeans,
            ),
            field(
                "BT-83",
                "Verwendungszweck",
                &[PAYMENT_MEANS, cbc("PaymentID")],
                Format::Text,
            ),
            field(
                "BT-84",
                "IBAN",
                &[PAYMENT_MEANS, PAYEE_ACCOUNT, cbc("ID")],
                Format::Iban,
            ),
            field(
                "BT-85",
                "Kontoinhaber",
                &[PAYMENT_MEANS, PAYEE_ACCOUNT, cbc("Name")],
                Format::Text,
            ),
            field(
                "BT-86",
                "BIC",
                &[
                    PAYMENT_MEANS,
                    PAYEE_ACCOUNT,
                    cac("FinancialInstitutionBranch"),
                    cbc("ID"),
                ],
                Format::Text,
            ),
            field(
                "BT-20",
                "Zahlungsbedingungen",
                &[cac("PaymentTerms"), cbc("Note")],
                Format::Text,
            ),
        ],
    },
    SectionDef {
        title: "Umsatzsteuer (BG-23)",
        anchor: &[cac("TaxTotal"), cac("TaxSubtotal")],
        repeatable: true,
        title_suffix: None,
        fields: &[
            field(
                "BT-116",
                "Steuerbasisbetrag",
                &[cbc("TaxableAmount")],
                Format::Amount,
            ),
            field(
                "BT-117",
                "Steuerbetrag",
                &[cbc("TaxAmount")],
                Format::Amount,
            ),
            field(
                "BT-118",
                "Steuerkategorie",
                &[cac("TaxCategory"), cbc("ID")],
                Format::VatCategory,
            ),
            field(
                "BT-119",
                "Steuersatz",
                &[cac("TaxCategory"), cbc("Percent")],
                Format::Text,
            ),
            field(
                "BT-120",
                "Befreiungsgrund",
                &[cac("TaxCategory"), cbc("TaxExemptionReason")],
                Format::Text,
            ),
        ],
    },
    SectionDef {
        title: "Summen (BG-22)",
        anchor: &[MONETARY_TOTAL],
        repeatable: false,
        title_suffix: None,
        fields: &[
            field(
                "BT-106",
                "Summe der Positionen netto",
                &[cbc("LineExtensionAmount")],
                Format::Amount,
            ),
            field(
                "BT-107",
                "Summe der Nachlässe",
                &[cbc("AllowanceTotalAmount")],
                Format::Amount,
            ),
            field(
                "BT-108",
                "Summe der Zuschläge",
                &[cbc("ChargeTotalAmount")],
                Format::Amount,
            ),
            field(
                "BT-109",
                "Gesamtbetrag netto",
                &[cbc("TaxExclusiveAmount")],
                Format::Amount,
            ),
            // The total VAT amount is outside the monetary total.
            absolute(
                "BT-110",
                "Umsatzsteuer gesamt",
                &[cac("TaxTotal"), cbc("TaxAmount")],
                Format::Amount,
            ),
            field(
                "BT-112",
                "Rechnungsbetrag inkl. USt",
                &[cbc("TaxInclusiveAmount")],
                Format::Amount,
            ),
            field(
                "BT-113",
                "Vorauszahlung",
                &[cbc("PrepaidAmount")],
                Format::Amount,
            ),
            field(
                "BT-114",
                "Rundungsbetrag",
                &[cbc("PayableRoundingAmount")],
                Format::Amount,
            ),
            field(
                "BT-115",
                "Zahlbetrag",
                &[cbc("PayableAmount")],
                Format::Amount,
            ),
        ],
    },
];
