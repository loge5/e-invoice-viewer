//! Where the business terms live in a UBL invoice or credit note (spec 0006).
//!
//! Sections, field ids, labels and display rules are the same as for CII
//! (`cii.rs`); only the paths differ. Paths follow the UBL syntax binding of
//! EN 16931.

use super::other::FixedValue;
use super::terms::*;
use super::{
    Condition, Format, SectionDef, Step, SupportingDocumentsDef, SyntaxDef, absolute, field,
    from_attribute, group, named, repeatable,
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
    fixed_values: &[
        // The tax scheme of every tax category is VAT.
        FixedValue {
            parent: cac("TaxScheme"),
            element: cbc("ID"),
            value: "VAT",
        },
        // The document type of an invoiced object identifier (BT-128).
        FixedValue {
            parent: cac("DocumentReference"),
            element: cbc("DocumentTypeCode"),
            value: "130",
        },
    ],
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
/// A party identification that is not the SEPA creditor identifier (BT-90).
const PARTY_ID: [Step; 2] = [
    cac("PartyIdentification"),
    cbc("ID").with(Condition::NotAttribute("schemeID", "SEPA")),
];
/// The SEPA creditor identifier (BT-90) among the party identifications.
const SEPA_ID: [Step; 2] = [
    cac("PartyIdentification"),
    cbc("ID").with_attribute("schemeID", "SEPA"),
];
const ALLOWANCE_CHARGE: Step = cac("AllowanceCharge");
const CHARGE_INDICATOR: &[Step] = &[cbc("ChargeIndicator")];
const PERIOD: Step = cac("InvoicePeriod");
const DELIVERY_ADDRESS: [Step; 2] = [cac("DeliveryLocation"), cac("Address")];
const ADDRESS_LINE: [Step; 2] = [cac("AddressLine"), cbc("Line")];
const LEGAL_ENTITY: Step = cac("PartyLegalEntity");
const CONTACT: Step = cac("Contact");

const HEADER_SECTIONS: &[SectionDef] = &[
    SectionDef {
        title: INVOICE,
        anchor: &[],
        repeatable: false,
        fallback: None,
        children: &[],
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
            field(BT_6, &[cbc("TaxCurrencyCode")], Format::Text),
            field(BT_7, &[cbc("TaxPointDate")], Format::IsoDate),
            field(BT_8, &[PERIOD, cbc("DescriptionCode")], Format::Text),
            // Invoice: cbc:DueDate; credit note: in the payment means.
            field(BT_9, &[cbc("DueDate")], Format::IsoDate),
            field(
                BT_9,
                &[cac("PaymentMeans"), cbc("PaymentDueDate")],
                Format::IsoDate,
            ),
            field(BT_10, &[cbc("BuyerReference")], Format::Text),
            field(BT_11, &[cac("ProjectReference"), cbc("ID")], Format::Text),
            field(
                BT_12,
                &[cac("ContractDocumentReference"), cbc("ID")],
                Format::Text,
            ),
            field(BT_13, &[cac("OrderReference"), cbc("ID")], Format::Text),
            field(
                BT_14,
                &[cac("OrderReference"), cbc("SalesOrderID")],
                Format::Text,
            ),
            field(
                BT_15,
                &[cac("ReceiptDocumentReference"), cbc("ID")],
                Format::Text,
            ),
            field(
                BT_16,
                &[cac("DespatchDocumentReference"), cbc("ID")],
                Format::Text,
            ),
            field(
                BT_17,
                &[cac("OriginatorDocumentReference"), cbc("ID")],
                Format::Text,
            ),
            field(
                BT_18,
                &[
                    cac("AdditionalDocumentReference")
                        .with(Condition::ChildText(&[cbc("DocumentTypeCode")], "130")),
                    cbc("ID"),
                ],
                Format::Text,
            ),
            field(BT_19, &[cbc("AccountingCost")], Format::Text),
            field(BT_23, &[cbc("ProfileID")], Format::Text),
            field(
                BT_24,
                &[cbc("CustomizationID")],
                Format::SpecificationIdentifier,
            ),
            // UBL writes the subject code (BT-21) into the note as `#AAI#…`.
            group(
                &[cbc("Note")],
                &[
                    field(BT_21, &[], Format::NoteSubject),
                    field(BT_22, &[], Format::NoteText),
                ],
            ),
        ],
    },
    SectionDef {
        title: PRECEDING_INVOICE,
        anchor: &[cac("BillingReference"), cac("InvoiceDocumentReference")],
        fallback: None,
        children: &[],
        repeatable: true,
        title_suffix: None,
        fields: &[
            field(BT_25, &[cbc("ID")], Format::Text),
            field(BT_26, &[cbc("IssueDate")], Format::IsoDate),
        ],
    },
    SectionDef {
        title: SELLER,
        anchor: &[cac("AccountingSupplierParty"), cac("Party")],
        repeatable: false,
        fallback: None,
        children: &[],
        title_suffix: None,
        fields: &[
            field(
                BT_27,
                &[LEGAL_ENTITY, cbc("RegistrationName")],
                Format::Text,
            ),
            field(BT_28, &[cac("PartyName"), cbc("Name")], Format::Text),
            repeatable(BT_29, &PARTY_ID, Format::Text),
            field(
                BT_30,
                &[LEGAL_ENTITY, cbc("CompanyID")],
                Format::Scheme("schemeID"),
            ),
            field(BT_31, &[VAT_SCHEME, cbc("CompanyID")], Format::Text),
            field(BT_32, &[OTHER_TAX_SCHEME, cbc("CompanyID")], Format::Text),
            field(
                BT_33,
                &[LEGAL_ENTITY, cbc("CompanyLegalForm")],
                Format::Text,
            ),
            field(BT_34, &[cbc("EndpointID")], Format::Scheme("schemeID")),
            field(BT_35, &[ADDRESS, cbc("StreetName")], Format::Text),
            field(BT_36, &[ADDRESS, cbc("AdditionalStreetName")], Format::Text),
            field(
                BT_162,
                &[ADDRESS, ADDRESS_LINE[0], ADDRESS_LINE[1]],
                Format::Text,
            ),
            field(BT_38, &[ADDRESS, cbc("PostalZone")], Format::Text),
            field(BT_37, &[ADDRESS, cbc("CityName")], Format::Text),
            field(BT_39, &[ADDRESS, cbc("CountrySubentity")], Format::Text),
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
        fallback: None,
        children: &[],
        title_suffix: None,
        fields: &[
            field(
                BT_44,
                &[LEGAL_ENTITY, cbc("RegistrationName")],
                Format::Text,
            ),
            field(BT_45, &[cac("PartyName"), cbc("Name")], Format::Text),
            field(BT_46, &PARTY_ID, Format::Text),
            field(
                BT_47,
                &[LEGAL_ENTITY, cbc("CompanyID")],
                Format::Scheme("schemeID"),
            ),
            field(BT_48, &[VAT_SCHEME, cbc("CompanyID")], Format::Text),
            field(BT_49, &[cbc("EndpointID")], Format::Scheme("schemeID")),
            field(BT_50, &[ADDRESS, cbc("StreetName")], Format::Text),
            field(BT_51, &[ADDRESS, cbc("AdditionalStreetName")], Format::Text),
            field(
                BT_163,
                &[ADDRESS, ADDRESS_LINE[0], ADDRESS_LINE[1]],
                Format::Text,
            ),
            field(BT_53, &[ADDRESS, cbc("PostalZone")], Format::Text),
            field(BT_52, &[ADDRESS, cbc("CityName")], Format::Text),
            field(BT_54, &[ADDRESS, cbc("CountrySubentity")], Format::Text),
            field(
                BT_55,
                &[ADDRESS, cac("Country"), cbc("IdentificationCode")],
                Format::Text,
            ),
            field(BT_56, &[CONTACT, cbc("Name")], Format::Text),
            field(BT_57, &[CONTACT, cbc("Telephone")], Format::Text),
            field(BT_58, &[CONTACT, cbc("ElectronicMail")], Format::Text),
        ],
    },
    SectionDef {
        title: PAYEE,
        anchor: &[cac("PayeeParty")],
        repeatable: false,
        fallback: None,
        children: &[],
        title_suffix: None,
        fields: &[
            field(BT_59, &[cac("PartyName"), cbc("Name")], Format::Text),
            field(BT_60, &PARTY_ID, Format::Scheme("schemeID")),
            field(
                BT_61,
                &[LEGAL_ENTITY, cbc("CompanyID")],
                Format::Scheme("schemeID"),
            ),
        ],
    },
    SectionDef {
        title: TAX_REPRESENTATIVE,
        anchor: &[cac("TaxRepresentativeParty")],
        repeatable: false,
        fallback: None,
        children: &[],
        title_suffix: None,
        fields: &[
            field(BT_62, &[cac("PartyName"), cbc("Name")], Format::Text),
            field(BT_63, &[VAT_SCHEME, cbc("CompanyID")], Format::Text),
            field(BT_64, &[ADDRESS, cbc("StreetName")], Format::Text),
            field(BT_65, &[ADDRESS, cbc("AdditionalStreetName")], Format::Text),
            field(
                BT_164,
                &[ADDRESS, ADDRESS_LINE[0], ADDRESS_LINE[1]],
                Format::Text,
            ),
            field(BT_67, &[ADDRESS, cbc("PostalZone")], Format::Text),
            field(BT_66, &[ADDRESS, cbc("CityName")], Format::Text),
            field(BT_68, &[ADDRESS, cbc("CountrySubentity")], Format::Text),
            field(
                BT_69,
                &[ADDRESS, cac("Country"), cbc("IdentificationCode")],
                Format::Text,
            ),
        ],
    },
    SectionDef {
        title: DELIVERY,
        // cac:Delivery is optional; an invoicing period alone still makes
        // the section (spec 0021).
        anchor: &[cac("Delivery")],
        fallback: Some(&[PERIOD]),
        children: &[],
        repeatable: false,
        title_suffix: None,
        fields: &[
            field(
                BT_70,
                &[cac("DeliveryParty"), cac("PartyName"), cbc("Name")],
                Format::Text,
            ),
            field(BT_71, &[cac("DeliveryLocation"), cbc("ID")], Format::Text),
            field(BT_72, &[cbc("ActualDeliveryDate")], Format::IsoDate),
            absolute(BT_73, &[PERIOD, cbc("StartDate")], Format::IsoDate),
            absolute(BT_74, &[PERIOD, cbc("EndDate")], Format::IsoDate),
            field(
                BT_75,
                &[DELIVERY_ADDRESS[0], DELIVERY_ADDRESS[1], cbc("StreetName")],
                Format::Text,
            ),
            field(
                BT_76,
                &[
                    DELIVERY_ADDRESS[0],
                    DELIVERY_ADDRESS[1],
                    cbc("AdditionalStreetName"),
                ],
                Format::Text,
            ),
            field(
                BT_165,
                &[
                    DELIVERY_ADDRESS[0],
                    DELIVERY_ADDRESS[1],
                    ADDRESS_LINE[0],
                    ADDRESS_LINE[1],
                ],
                Format::Text,
            ),
            field(
                BT_78,
                &[DELIVERY_ADDRESS[0], DELIVERY_ADDRESS[1], cbc("PostalZone")],
                Format::Text,
            ),
            field(
                BT_77,
                &[DELIVERY_ADDRESS[0], DELIVERY_ADDRESS[1], cbc("CityName")],
                Format::Text,
            ),
            field(
                BT_79,
                &[
                    DELIVERY_ADDRESS[0],
                    DELIVERY_ADDRESS[1],
                    cbc("CountrySubentity"),
                ],
                Format::Text,
            ),
            field(
                BT_80,
                &[
                    DELIVERY_ADDRESS[0],
                    DELIVERY_ADDRESS[1],
                    cac("Country"),
                    cbc("IdentificationCode"),
                ],
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
    fallback: None,
    children: LINE_CHILDREN,
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
        field(
            BT_156,
            &[ITEM, cac("BuyersItemIdentification"), cbc("ID")],
            Format::Text,
        ),
        field(
            BT_157,
            &[ITEM, cac("StandardItemIdentification"), cbc("ID")],
            Format::Scheme("schemeID"),
        ),
        repeatable(
            BT_158,
            &[
                ITEM,
                cac("CommodityClassification"),
                cbc("ItemClassificationCode"),
            ],
            Format::Scheme("listID"),
        ),
        field(
            BT_159,
            &[ITEM, cac("OriginCountry"), cbc("IdentificationCode")],
            Format::Text,
        ),
        field(BT_129, &QUANTITY, Format::Quantity),
        from_attribute(BT_130, &QUANTITY, "unitCode"),
        field(
            BT_148,
            &[cac("Price"), PRICE_DISCOUNT, cbc("BaseAmount")],
            Format::Amount,
        ),
        field(
            BT_147,
            &[cac("Price"), PRICE_DISCOUNT, cbc("Amount")],
            Format::Amount,
        ),
        field(BT_146, &[cac("Price"), cbc("PriceAmount")], Format::Amount),
        field(
            BT_149,
            &[cac("Price"), cbc("BaseQuantity")],
            Format::Quantity,
        ),
        from_attribute(BT_150, &[cac("Price"), cbc("BaseQuantity")], "unitCode"),
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
        field(
            BT_128,
            &[cac("DocumentReference"), cbc("ID")],
            Format::Scheme("schemeID"),
        ),
        field(
            BT_132,
            &[cac("OrderLineReference"), cbc("LineID")],
            Format::Text,
        ),
        field(BT_133, &[cbc("AccountingCost")], Format::Text),
        field(BT_134, &[PERIOD, cbc("StartDate")], Format::IsoDate),
        field(BT_135, &[PERIOD, cbc("EndDate")], Format::IsoDate),
        // Item attributes (BG-32): one field per attribute, labelled with its
        // name (BT-160), showing its value (BT-161).
        group(
            &[ITEM, cac("AdditionalItemProperty")],
            &[named(BT_161, &[cbc("Value")], &[cbc("Name")])],
        ),
    ],
}];

/// The item price discount: the allowance of the price (BT-147, BT-148).
const PRICE_DISCOUNT: Step = cac("AllowanceCharge");

/// Allowances (BG-27) and charges (BG-28) of an invoice line, after it.
const LINE_CHILDREN: &[SectionDef] = &[
    SectionDef {
        title: LINE_ALLOWANCE,
        anchor: &[ALLOWANCE_CHARGE.with(Condition::ChildText(CHARGE_INDICATOR, "false"))],
        repeatable: true,
        fallback: None,
        children: &[],
        title_suffix: None,
        fields: &[
            field(BT_136, &[cbc("Amount")], Format::Amount),
            field(BT_137, &[cbc("BaseAmount")], Format::Amount),
            field(BT_138, &[cbc("MultiplierFactorNumeric")], Format::Text),
            field(BT_139, &[cbc("AllowanceChargeReason")], Format::Text),
            field(BT_140, &[cbc("AllowanceChargeReasonCode")], Format::Text),
        ],
    },
    SectionDef {
        title: LINE_CHARGE,
        anchor: &[ALLOWANCE_CHARGE.with(Condition::ChildText(CHARGE_INDICATOR, "true"))],
        repeatable: true,
        fallback: None,
        children: &[],
        title_suffix: None,
        fields: &[
            field(BT_141, &[cbc("Amount")], Format::Amount),
            field(BT_142, &[cbc("BaseAmount")], Format::Amount),
            field(BT_143, &[cbc("MultiplierFactorNumeric")], Format::Text),
            field(BT_144, &[cbc("AllowanceChargeReason")], Format::Text),
            field(BT_145, &[cbc("AllowanceChargeReasonCode")], Format::Text),
        ],
    },
];

const PAYMENT_MEANS: Step = cac("PaymentMeans");
const PAYEE_ACCOUNT: Step = cac("PayeeFinancialAccount");
const MONETARY_TOTAL: Step = cac("LegalMonetaryTotal");

const TRAILER_SECTIONS: &[SectionDef] = &[
    SectionDef {
        title: PAYMENT,
        // The payment terms of UBL are outside the payment means.
        anchor: &[],
        repeatable: false,
        fallback: None,
        children: &[],
        title_suffix: None,
        fields: &[
            field(
                BT_81,
                &[PAYMENT_MEANS, cbc("PaymentMeansCode")],
                Format::PaymentMeans,
            ),
            from_attribute(BT_82, &[PAYMENT_MEANS, cbc("PaymentMeansCode")], "name"),
            field(BT_83, &[PAYMENT_MEANS, cbc("PaymentID")], Format::Text),
            // Credit transfer (BG-17) may occur more than once, one per
            // payment means.
            group(
                &[PAYMENT_MEANS],
                &[
                    field(BT_84, &[PAYEE_ACCOUNT, cbc("ID")], Format::Iban),
                    field(BT_85, &[PAYEE_ACCOUNT, cbc("Name")], Format::Text),
                    field(
                        BT_86,
                        &[PAYEE_ACCOUNT, cac("FinancialInstitutionBranch"), cbc("ID")],
                        Format::Text,
                    ),
                ],
            ),
            field(
                BT_87,
                &[
                    PAYMENT_MEANS,
                    cac("CardAccount"),
                    cbc("PrimaryAccountNumberID"),
                ],
                Format::Text,
            ),
            field(
                BT_88,
                &[PAYMENT_MEANS, cac("CardAccount"), cbc("HolderName")],
                Format::Text,
            ),
            field(
                BT_89,
                &[PAYMENT_MEANS, cac("PaymentMandate"), cbc("ID")],
                Format::Text,
            ),
            // The creditor identifier is a party identification of the
            // seller, or of the payee if there is one.
            field(
                BT_90,
                &[
                    cac("AccountingSupplierParty"),
                    cac("Party"),
                    SEPA_ID[0],
                    SEPA_ID[1],
                ],
                Format::Text,
            ),
            field(
                BT_90,
                &[cac("PayeeParty"), SEPA_ID[0], SEPA_ID[1]],
                Format::Text,
            ),
            field(
                BT_91,
                &[
                    PAYMENT_MEANS,
                    cac("PaymentMandate"),
                    cac("PayerFinancialAccount"),
                    cbc("ID"),
                ],
                Format::Iban,
            ),
            field(BT_20, &[cac("PaymentTerms"), cbc("Note")], Format::Text),
        ],
    },
    SectionDef {
        title: ALLOWANCE,
        anchor: &[ALLOWANCE_CHARGE.with(Condition::ChildText(CHARGE_INDICATOR, "false"))],
        fallback: None,
        children: &[],
        repeatable: true,
        title_suffix: None,
        fields: &[
            field(BT_92, &[cbc("Amount")], Format::Amount),
            field(BT_93, &[cbc("BaseAmount")], Format::Amount),
            field(BT_94, &[cbc("MultiplierFactorNumeric")], Format::Text),
            field(BT_95, &[cac("TaxCategory"), cbc("ID")], Format::VatCategory),
            field(BT_96, &[cac("TaxCategory"), cbc("Percent")], Format::Text),
            field(BT_97, &[cbc("AllowanceChargeReason")], Format::Text),
            field(BT_98, &[cbc("AllowanceChargeReasonCode")], Format::Text),
        ],
    },
    SectionDef {
        title: CHARGE,
        anchor: &[ALLOWANCE_CHARGE.with(Condition::ChildText(CHARGE_INDICATOR, "true"))],
        fallback: None,
        children: &[],
        repeatable: true,
        title_suffix: None,
        fields: &[
            field(BT_99, &[cbc("Amount")], Format::Amount),
            field(BT_100, &[cbc("BaseAmount")], Format::Amount),
            field(BT_101, &[cbc("MultiplierFactorNumeric")], Format::Text),
            field(
                BT_102,
                &[cac("TaxCategory"), cbc("ID")],
                Format::VatCategory,
            ),
            field(BT_103, &[cac("TaxCategory"), cbc("Percent")], Format::Text),
            field(BT_104, &[cbc("AllowanceChargeReason")], Format::Text),
            field(BT_105, &[cbc("AllowanceChargeReasonCode")], Format::Text),
        ],
    },
    SectionDef {
        title: VAT_BREAKDOWN,
        anchor: &[cac("TaxTotal"), cac("TaxSubtotal")],
        repeatable: true,
        fallback: None,
        children: &[],
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
            field(
                BT_121,
                &[cac("TaxCategory"), cbc("TaxExemptionReasonCode")],
                Format::Text,
            ),
        ],
    },
    SectionDef {
        title: TOTALS,
        anchor: &[MONETARY_TOTAL],
        repeatable: false,
        fallback: None,
        children: &[],
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
            absolute(
                BT_111,
                &[
                    cac("TaxTotal").with(Condition::NotChild(&[cac("TaxSubtotal")])),
                    cbc("TaxAmount"),
                ],
                Format::OwnCurrencyAmount,
            ),
            field(BT_112, &[cbc("TaxInclusiveAmount")], Format::Amount),
            field(BT_113, &[cbc("PrepaidAmount")], Format::Amount),
            field(BT_114, &[cbc("PayableRoundingAmount")], Format::Amount),
            field(BT_115, &[cbc("PayableAmount")], Format::Amount),
        ],
    },
];
