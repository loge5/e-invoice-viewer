//! Where the business terms live in a CII invoice (spec 0004).

use super::{FieldDef, Format, SectionDef, Step, SyntaxDef};

const RSM: &str = "urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100";
const RAM: &str =
    "urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100";
const UDT: &str = "urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100";

/// Namespace prefixes used in field locations.
const PREFIXES: &[(&str, &str)] = &[
    ("rsm", RSM),
    ("ram", RAM),
    ("udt", UDT),
    (
        "qdt",
        "urn:un:unece:uncefact:data:standard:QualifiedDataType:100",
    ),
];

const fn rsm(name: &'static str) -> Step {
    Step::new(RSM, name)
}

const fn ram(name: &'static str) -> Step {
    Step::new(RAM, name)
}

const fn udt(name: &'static str) -> Step {
    Step::new(UDT, name)
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
        id,
        label,
        path,
        format,
        repeatable: true,
        attribute: None,
        absolute: false,
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
        id,
        label,
        path,
        format: Format::Text,
        repeatable: false,
        attribute: Some(attribute),
        absolute: false,
    }
}

const SETTLEMENT: [Step; 2] = [
    rsm("SupplyChainTradeTransaction"),
    ram("ApplicableHeaderTradeSettlement"),
];

/// The CII mapping.
pub const SYNTAX: SyntaxDef = SyntaxDef {
    prefixes: PREFIXES,
    currency_path: CURRENCY_PATH,
    sections: &[HEADER_SECTIONS, LINE_SECTIONS, TRAILER_SECTIONS],
};

/// Path from the root to the invoice currency (BT-5), used for amounts.
const CURRENCY_PATH: &[Step] = &[SETTLEMENT[0], SETTLEMENT[1], ram("InvoiceCurrencyCode")];

const ADDRESS: Step = ram("PostalTradeAddress");
const TAX_REGISTRATION: Step = ram("SpecifiedTaxRegistration");

/// Header sections in display order; line items (0005) are inserted before
/// the payment section.
const HEADER_SECTIONS: &[SectionDef] = &[
    SectionDef {
        title: "Rechnung",
        anchor: &[],
        repeatable: false,
        title_suffix: None,
        fields: &[
            field(
                "BT-1",
                "Rechnungsnummer",
                &[rsm("ExchangedDocument"), ram("ID")],
                Format::Text,
            ),
            field(
                "BT-2",
                "Rechnungsdatum",
                &[
                    rsm("ExchangedDocument"),
                    ram("IssueDateTime"),
                    udt("DateTimeString"),
                ],
                Format::Date,
            ),
            field(
                "BT-3",
                "Rechnungsart",
                &[rsm("ExchangedDocument"), ram("TypeCode")],
                Format::InvoiceType,
            ),
            field("BT-5", "Währung", CURRENCY_PATH, Format::Text),
            field(
                "BT-9",
                "Fälligkeitsdatum",
                &[
                    SETTLEMENT[0],
                    SETTLEMENT[1],
                    ram("SpecifiedTradePaymentTerms"),
                    ram("DueDateDateTime"),
                    udt("DateTimeString"),
                ],
                Format::Date,
            ),
            field(
                "BT-10",
                "Käuferreferenz",
                &[
                    rsm("SupplyChainTradeTransaction"),
                    ram("ApplicableHeaderTradeAgreement"),
                    ram("BuyerReference"),
                ],
                Format::Text,
            ),
            field(
                "BT-24",
                "Spezifikationskennung",
                &[
                    rsm("ExchangedDocumentContext"),
                    ram("GuidelineSpecifiedDocumentContextParameter"),
                    ram("ID"),
                ],
                Format::SpecificationIdentifier,
            ),
            repeatable(
                "BT-22",
                "Bemerkung",
                &[
                    rsm("ExchangedDocument"),
                    ram("IncludedNote"),
                    ram("Content"),
                ],
                Format::Text,
            ),
        ],
    },
    SectionDef {
        title: "Verkäufer (BG-4)",
        anchor: &[
            rsm("SupplyChainTradeTransaction"),
            ram("ApplicableHeaderTradeAgreement"),
            ram("SellerTradeParty"),
        ],
        repeatable: false,
        title_suffix: None,
        fields: &[
            field("BT-27", "Name", &[ram("Name")], Format::Text),
            field("BT-29", "Kennung", &[ram("ID")], Format::Text),
            field(
                "BT-31",
                "Umsatzsteuer-ID",
                &[TAX_REGISTRATION, ram("ID").with_attribute("schemeID", "VA")],
                Format::Text,
            ),
            field(
                "BT-32",
                "Steuernummer",
                &[TAX_REGISTRATION, ram("ID").with_attribute("schemeID", "FC")],
                Format::Text,
            ),
            field("BT-35", "Straße", &[ADDRESS, ram("LineOne")], Format::Text),
            field(
                "BT-36",
                "Adresszusatz",
                &[ADDRESS, ram("LineTwo")],
                Format::Text,
            ),
            field(
                "BT-38",
                "PLZ",
                &[ADDRESS, ram("PostcodeCode")],
                Format::Text,
            ),
            field("BT-37", "Ort", &[ADDRESS, ram("CityName")], Format::Text),
            field("BT-40", "Land", &[ADDRESS, ram("CountryID")], Format::Text),
            field(
                "BT-41",
                "Kontaktperson",
                &[ram("DefinedTradeContact"), ram("PersonName")],
                Format::Text,
            ),
            field(
                "BT-42",
                "Telefon",
                &[
                    ram("DefinedTradeContact"),
                    ram("TelephoneUniversalCommunication"),
                    ram("CompleteNumber"),
                ],
                Format::Text,
            ),
            field(
                "BT-43",
                "E-Mail",
                &[
                    ram("DefinedTradeContact"),
                    ram("EmailURIUniversalCommunication"),
                    ram("URIID"),
                ],
                Format::Text,
            ),
        ],
    },
    SectionDef {
        title: "Käufer (BG-7)",
        anchor: &[
            rsm("SupplyChainTradeTransaction"),
            ram("ApplicableHeaderTradeAgreement"),
            ram("BuyerTradeParty"),
        ],
        repeatable: false,
        title_suffix: None,
        fields: &[
            field("BT-44", "Name", &[ram("Name")], Format::Text),
            field("BT-46", "Kennung", &[ram("ID")], Format::Text),
            field(
                "BT-48",
                "Umsatzsteuer-ID",
                &[TAX_REGISTRATION, ram("ID").with_attribute("schemeID", "VA")],
                Format::Text,
            ),
            field("BT-50", "Straße", &[ADDRESS, ram("LineOne")], Format::Text),
            field(
                "BT-51",
                "Adresszusatz",
                &[ADDRESS, ram("LineTwo")],
                Format::Text,
            ),
            field(
                "BT-53",
                "PLZ",
                &[ADDRESS, ram("PostcodeCode")],
                Format::Text,
            ),
            field("BT-52", "Ort", &[ADDRESS, ram("CityName")], Format::Text),
            field("BT-55", "Land", &[ADDRESS, ram("CountryID")], Format::Text),
        ],
    },
];

const LINE_DOCUMENT: Step = ram("AssociatedDocumentLineDocument");
const PRODUCT: Step = ram("SpecifiedTradeProduct");
const NET_PRICE: [Step; 2] = [
    ram("SpecifiedLineTradeAgreement"),
    ram("NetPriceProductTradePrice"),
];
const LINE_SETTLEMENT: Step = ram("SpecifiedLineTradeSettlement");
const BILLED_QUANTITY: [Step; 2] = [ram("SpecifiedLineTradeDelivery"), ram("BilledQuantity")];

/// One section per invoice line (BG-25), between buyer and payment (0005).
const LINE_SECTIONS: &[SectionDef] = &[SectionDef {
    title: "Position",
    anchor: &[
        rsm("SupplyChainTradeTransaction"),
        ram("IncludedSupplyChainTradeLineItem"),
    ],
    repeatable: true,
    title_suffix: Some(&[LINE_DOCUMENT, ram("LineID")]),
    fields: &[
        field(
            "BT-126",
            "Positionsnummer",
            &[LINE_DOCUMENT, ram("LineID")],
            Format::Text,
        ),
        repeatable(
            "BT-127",
            "Positionsbemerkung",
            &[LINE_DOCUMENT, ram("IncludedNote"), ram("Content")],
            Format::Text,
        ),
        field(
            "BT-153",
            "Artikelname",
            &[PRODUCT, ram("Name")],
            Format::Text,
        ),
        field(
            "BT-154",
            "Artikelbeschreibung",
            &[PRODUCT, ram("Description")],
            Format::Text,
        ),
        field(
            "BT-155",
            "Artikelnummer",
            &[PRODUCT, ram("SellerAssignedID")],
            Format::Text,
        ),
        field("BT-129", "Menge", &BILLED_QUANTITY, Format::Quantity),
        from_attribute("BT-130", "Einheit", &BILLED_QUANTITY, "unitCode"),
        field(
            "BT-146",
            "Nettopreis",
            &[NET_PRICE[0], NET_PRICE[1], ram("ChargeAmount")],
            Format::Amount,
        ),
        field(
            "BT-149",
            "Basismenge",
            &[NET_PRICE[0], NET_PRICE[1], ram("BasisQuantity")],
            Format::Quantity,
        ),
        field(
            "BT-131",
            "Nettobetrag",
            &[
                LINE_SETTLEMENT,
                ram("SpecifiedTradeSettlementLineMonetarySummation"),
                ram("LineTotalAmount"),
            ],
            Format::Amount,
        ),
        field(
            "BT-151",
            "Steuerkategorie",
            &[
                LINE_SETTLEMENT,
                ram("ApplicableTradeTax"),
                ram("CategoryCode"),
            ],
            Format::VatCategory,
        ),
        field(
            "BT-152",
            "Steuersatz",
            &[
                LINE_SETTLEMENT,
                ram("ApplicableTradeTax"),
                ram("RateApplicablePercent"),
            ],
            Format::Text,
        ),
        field(
            "BT-128",
            "Objektkennung",
            &[
                LINE_SETTLEMENT,
                ram("AdditionalReferencedDocument"),
                ram("IssuerAssignedID"),
            ],
            Format::Text,
        ),
        field(
            "BT-133",
            "Buchungsreferenz des Käufers",
            &[
                LINE_SETTLEMENT,
                ram("ReceivableSpecifiedTradeAccountingAccount"),
                ram("ID"),
            ],
            Format::Text,
        ),
    ],
}];

/// Sections after the line items.
const TRAILER_SECTIONS: &[SectionDef] = &[
    SectionDef {
        title: "Zahlung (BG-16)",
        anchor: &[SETTLEMENT[0], SETTLEMENT[1]],
        repeatable: false,
        title_suffix: None,
        fields: &[
            field(
                "BT-81",
                "Zahlungsart",
                &[ram("SpecifiedTradeSettlementPaymentMeans"), ram("TypeCode")],
                Format::PaymentMeans,
            ),
            field(
                "BT-83",
                "Verwendungszweck",
                &[ram("PaymentReference")],
                Format::Text,
            ),
            field(
                "BT-84",
                "IBAN",
                &[
                    ram("SpecifiedTradeSettlementPaymentMeans"),
                    ram("PayeePartyCreditorFinancialAccount"),
                    ram("IBANID"),
                ],
                Format::Iban,
            ),
            field(
                "BT-85",
                "Kontoinhaber",
                &[
                    ram("SpecifiedTradeSettlementPaymentMeans"),
                    ram("PayeePartyCreditorFinancialAccount"),
                    ram("AccountName"),
                ],
                Format::Text,
            ),
            field(
                "BT-86",
                "BIC",
                &[
                    ram("SpecifiedTradeSettlementPaymentMeans"),
                    ram("PayeeSpecifiedCreditorFinancialInstitution"),
                    ram("BICID"),
                ],
                Format::Text,
            ),
            field(
                "BT-20",
                "Zahlungsbedingungen",
                &[ram("SpecifiedTradePaymentTerms"), ram("Description")],
                Format::Text,
            ),
        ],
    },
    SectionDef {
        title: "Umsatzsteuer (BG-23)",
        anchor: &[SETTLEMENT[0], SETTLEMENT[1], ram("ApplicableTradeTax")],
        repeatable: true,
        title_suffix: None,
        fields: &[
            field(
                "BT-116",
                "Steuerbasisbetrag",
                &[ram("BasisAmount")],
                Format::Amount,
            ),
            field(
                "BT-117",
                "Steuerbetrag",
                &[ram("CalculatedAmount")],
                Format::Amount,
            ),
            field(
                "BT-118",
                "Steuerkategorie",
                &[ram("CategoryCode")],
                Format::VatCategory,
            ),
            field(
                "BT-119",
                "Steuersatz",
                &[ram("RateApplicablePercent")],
                Format::Text,
            ),
            field(
                "BT-120",
                "Befreiungsgrund",
                &[ram("ExemptionReason")],
                Format::Text,
            ),
        ],
    },
    SectionDef {
        title: "Summen (BG-22)",
        anchor: &[
            SETTLEMENT[0],
            SETTLEMENT[1],
            ram("SpecifiedTradeSettlementHeaderMonetarySummation"),
        ],
        repeatable: false,
        title_suffix: None,
        fields: &[
            field(
                "BT-106",
                "Summe der Positionen netto",
                &[ram("LineTotalAmount")],
                Format::Amount,
            ),
            field(
                "BT-107",
                "Summe der Nachlässe",
                &[ram("AllowanceTotalAmount")],
                Format::Amount,
            ),
            field(
                "BT-108",
                "Summe der Zuschläge",
                &[ram("ChargeTotalAmount")],
                Format::Amount,
            ),
            field(
                "BT-109",
                "Gesamtbetrag netto",
                &[ram("TaxBasisTotalAmount")],
                Format::Amount,
            ),
            field(
                "BT-110",
                "Umsatzsteuer gesamt",
                &[ram("TaxTotalAmount")],
                Format::Amount,
            ),
            field(
                "BT-112",
                "Rechnungsbetrag inkl. USt",
                &[ram("GrandTotalAmount")],
                Format::Amount,
            ),
            field(
                "BT-113",
                "Vorauszahlung",
                &[ram("TotalPrepaidAmount")],
                Format::Amount,
            ),
            field(
                "BT-114",
                "Rundungsbetrag",
                &[ram("RoundingAmount")],
                Format::Amount,
            ),
            field(
                "BT-115",
                "Zahlbetrag",
                &[ram("DuePayableAmount")],
                Format::Amount,
            ),
        ],
    },
];
