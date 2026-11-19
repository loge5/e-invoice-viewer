//! Where the business terms live in a CII invoice (spec 0004).

use super::terms::*;
use super::{
    Condition, Format, SectionDef, Step, SupportingDocumentsDef, SyntaxDef, absolute, field,
    from_attribute, repeatable,
};
use crate::ns::{QDT, RAM, RSM, UDT};

/// Namespace prefixes used in field locations.
const PREFIXES: &[(&str, &str)] = &[("rsm", RSM), ("ram", RAM), ("udt", UDT), ("qdt", QDT)];

const fn rsm(name: &'static str) -> Step {
    Step::new(RSM, name)
}

const fn ram(name: &'static str) -> Step {
    Step::new(RAM, name)
}

const fn udt(name: &'static str) -> Step {
    Step::new(UDT, name)
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
    supporting_documents: SupportingDocumentsDef {
        // Type code 916: additional supporting document.
        anchor: &[
            rsm("SupplyChainTradeTransaction"),
            ram("ApplicableHeaderTradeAgreement"),
            ram("AdditionalReferencedDocument")
                .with(Condition::ChildText(&[ram("TypeCode")], "916")),
        ],
        binary: &[ram("AttachmentBinaryObject")],
        address: &[ram("URIID")],
        description: &[&[ram("Name")], &[ram("IssuerAssignedID")]],
    },
};

const AGREEMENT: [Step; 2] = [
    rsm("SupplyChainTradeTransaction"),
    ram("ApplicableHeaderTradeAgreement"),
];
const HEADER_DELIVERY: [Step; 2] = [
    rsm("SupplyChainTradeTransaction"),
    ram("ApplicableHeaderTradeDelivery"),
];
const ISSUER_ID: Step = ram("IssuerAssignedID");
const DATE: Step = udt("DateTimeString");
const ALLOWANCE_CHARGE: Step = ram("SpecifiedTradeAllowanceCharge");
const CHARGE_INDICATOR: &[Step] = &[ram("ChargeIndicator"), udt("Indicator")];
const ALLOWANCE_CATEGORY: Step = ram("CategoryTradeTax");

/// Path from the root to the invoice currency (BT-5), used for amounts.
const CURRENCY_PATH: &[Step] = &[SETTLEMENT[0], SETTLEMENT[1], ram("InvoiceCurrencyCode")];

const ADDRESS: Step = ram("PostalTradeAddress");
const TAX_REGISTRATION: Step = ram("SpecifiedTaxRegistration");

/// Header sections in display order; line items (0005) are inserted before
/// the payment section.
const HEADER_SECTIONS: &[SectionDef] = &[
    SectionDef {
        title: INVOICE,
        anchor: &[],
        repeatable: false,
        fallback: None,
        title_suffix: None,
        fields: &[
            field(BT_1, &[rsm("ExchangedDocument"), ram("ID")], Format::Text),
            field(
                BT_2,
                &[
                    rsm("ExchangedDocument"),
                    ram("IssueDateTime"),
                    udt("DateTimeString"),
                ],
                Format::Date,
            ),
            field(
                BT_3,
                &[rsm("ExchangedDocument"), ram("TypeCode")],
                Format::InvoiceType,
            ),
            field(BT_5, CURRENCY_PATH, Format::Text),
            field(
                BT_9,
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
                BT_10,
                &[
                    rsm("SupplyChainTradeTransaction"),
                    ram("ApplicableHeaderTradeAgreement"),
                    ram("BuyerReference"),
                ],
                Format::Text,
            ),
            field(
                BT_11,
                &[
                    AGREEMENT[0],
                    AGREEMENT[1],
                    ram("SpecifiedProcuringProject"),
                    ram("ID"),
                ],
                Format::Text,
            ),
            field(
                BT_12,
                &[
                    AGREEMENT[0],
                    AGREEMENT[1],
                    ram("ContractReferencedDocument"),
                    ISSUER_ID,
                ],
                Format::Text,
            ),
            field(
                BT_13,
                &[
                    AGREEMENT[0],
                    AGREEMENT[1],
                    ram("BuyerOrderReferencedDocument"),
                    ISSUER_ID,
                ],
                Format::Text,
            ),
            field(
                BT_14,
                &[
                    AGREEMENT[0],
                    AGREEMENT[1],
                    ram("SellerOrderReferencedDocument"),
                    ISSUER_ID,
                ],
                Format::Text,
            ),
            field(
                BT_15,
                &[
                    HEADER_DELIVERY[0],
                    HEADER_DELIVERY[1],
                    ram("ReceivingAdviceReferencedDocument"),
                    ISSUER_ID,
                ],
                Format::Text,
            ),
            field(
                BT_16,
                &[
                    HEADER_DELIVERY[0],
                    HEADER_DELIVERY[1],
                    ram("DespatchAdviceReferencedDocument"),
                    ISSUER_ID,
                ],
                Format::Text,
            ),
            field(
                BT_17,
                &[
                    AGREEMENT[0],
                    AGREEMENT[1],
                    ram("AdditionalReferencedDocument")
                        .with(Condition::ChildText(&[ram("TypeCode")], "50")),
                    ISSUER_ID,
                ],
                Format::Text,
            ),
            field(
                BT_18,
                &[
                    AGREEMENT[0],
                    AGREEMENT[1],
                    ram("AdditionalReferencedDocument")
                        .with(Condition::ChildText(&[ram("TypeCode")], "130")),
                    ISSUER_ID,
                ],
                Format::Text,
            ),
            field(
                BT_19,
                &[
                    SETTLEMENT[0],
                    SETTLEMENT[1],
                    ram("ReceivableSpecifiedTradeAccountingAccount"),
                    ram("ID"),
                ],
                Format::Text,
            ),
            field(
                BT_24,
                &[
                    rsm("ExchangedDocumentContext"),
                    ram("GuidelineSpecifiedDocumentContextParameter"),
                    ram("ID"),
                ],
                Format::SpecificationIdentifier,
            ),
            repeatable(
                BT_22,
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
        title: PRECEDING_INVOICE,
        anchor: &[
            SETTLEMENT[0],
            SETTLEMENT[1],
            ram("InvoiceReferencedDocument"),
        ],
        fallback: None,
        repeatable: true,
        title_suffix: None,
        fields: &[
            field(BT_25, &[ISSUER_ID], Format::Text),
            field(
                BT_26,
                &[
                    ram("FormattedIssueDateTime"),
                    Step::new(QDT, "DateTimeString"),
                ],
                Format::Date,
            ),
        ],
    },
    SectionDef {
        title: SELLER,
        anchor: &[
            rsm("SupplyChainTradeTransaction"),
            ram("ApplicableHeaderTradeAgreement"),
            ram("SellerTradeParty"),
        ],
        repeatable: false,
        fallback: None,
        title_suffix: None,
        fields: &[
            field(BT_27, &[ram("Name")], Format::Text),
            field(BT_29, &[ram("ID")], Format::Text),
            field(
                BT_31,
                &[TAX_REGISTRATION, ram("ID").with_attribute("schemeID", "VA")],
                Format::Text,
            ),
            field(
                BT_32,
                &[TAX_REGISTRATION, ram("ID").with_attribute("schemeID", "FC")],
                Format::Text,
            ),
            field(
                BT_34,
                &[ram("URIUniversalCommunication"), ram("URIID")],
                Format::ElectronicAddress,
            ),
            field(BT_35, &[ADDRESS, ram("LineOne")], Format::Text),
            field(BT_36, &[ADDRESS, ram("LineTwo")], Format::Text),
            field(BT_38, &[ADDRESS, ram("PostcodeCode")], Format::Text),
            field(BT_37, &[ADDRESS, ram("CityName")], Format::Text),
            field(BT_40, &[ADDRESS, ram("CountryID")], Format::Text),
            field(
                BT_41,
                &[ram("DefinedTradeContact"), ram("PersonName")],
                Format::Text,
            ),
            field(
                BT_42,
                &[
                    ram("DefinedTradeContact"),
                    ram("TelephoneUniversalCommunication"),
                    ram("CompleteNumber"),
                ],
                Format::Text,
            ),
            field(
                BT_43,
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
        title: BUYER,
        anchor: &[
            rsm("SupplyChainTradeTransaction"),
            ram("ApplicableHeaderTradeAgreement"),
            ram("BuyerTradeParty"),
        ],
        repeatable: false,
        fallback: None,
        title_suffix: None,
        fields: &[
            field(BT_44, &[ram("Name")], Format::Text),
            field(BT_46, &[ram("ID")], Format::Text),
            field(
                BT_48,
                &[TAX_REGISTRATION, ram("ID").with_attribute("schemeID", "VA")],
                Format::Text,
            ),
            field(
                BT_49,
                &[ram("URIUniversalCommunication"), ram("URIID")],
                Format::ElectronicAddress,
            ),
            field(BT_50, &[ADDRESS, ram("LineOne")], Format::Text),
            field(BT_51, &[ADDRESS, ram("LineTwo")], Format::Text),
            field(BT_53, &[ADDRESS, ram("PostcodeCode")], Format::Text),
            field(BT_52, &[ADDRESS, ram("CityName")], Format::Text),
            field(BT_55, &[ADDRESS, ram("CountryID")], Format::Text),
        ],
    },
    SectionDef {
        title: DELIVERY,
        anchor: &HEADER_DELIVERY,
        fallback: None,
        repeatable: false,
        title_suffix: None,
        fields: &[
            field(BT_70, &[SHIP_TO, ram("Name")], Format::Text),
            field(BT_71, &[SHIP_TO, ram("ID")], Format::Text),
            field(
                BT_72,
                &[
                    ram("ActualDeliverySupplyChainEvent"),
                    ram("OccurrenceDateTime"),
                    DATE,
                ],
                Format::Date,
            ),
            // The invoicing period (BG-14) belongs to BG-13 but is part of
            // the settlement in CII.
            absolute(
                BT_73,
                &[
                    SETTLEMENT[0],
                    SETTLEMENT[1],
                    PERIOD,
                    ram("StartDateTime"),
                    DATE,
                ],
                Format::Date,
            ),
            absolute(
                BT_74,
                &[
                    SETTLEMENT[0],
                    SETTLEMENT[1],
                    PERIOD,
                    ram("EndDateTime"),
                    DATE,
                ],
                Format::Date,
            ),
            field(BT_75, &[SHIP_TO, ADDRESS, ram("LineOne")], Format::Text),
            field(BT_76, &[SHIP_TO, ADDRESS, ram("LineTwo")], Format::Text),
            field(
                BT_78,
                &[SHIP_TO, ADDRESS, ram("PostcodeCode")],
                Format::Text,
            ),
            field(BT_77, &[SHIP_TO, ADDRESS, ram("CityName")], Format::Text),
            field(BT_80, &[SHIP_TO, ADDRESS, ram("CountryID")], Format::Text),
        ],
    },
];

const SHIP_TO: Step = ram("ShipToTradeParty");
const PERIOD: Step = ram("BillingSpecifiedPeriod");

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
    title: LINE,
    anchor: &[
        rsm("SupplyChainTradeTransaction"),
        ram("IncludedSupplyChainTradeLineItem"),
    ],
    repeatable: true,
    fallback: None,
    title_suffix: Some(&[LINE_DOCUMENT, ram("LineID")]),
    fields: &[
        field(BT_126, &[LINE_DOCUMENT, ram("LineID")], Format::Text),
        repeatable(
            BT_127,
            &[LINE_DOCUMENT, ram("IncludedNote"), ram("Content")],
            Format::Text,
        ),
        field(BT_153, &[PRODUCT, ram("Name")], Format::Text),
        field(BT_154, &[PRODUCT, ram("Description")], Format::Text),
        field(BT_155, &[PRODUCT, ram("SellerAssignedID")], Format::Text),
        field(BT_129, &BILLED_QUANTITY, Format::Quantity),
        from_attribute(BT_130, &BILLED_QUANTITY, "unitCode"),
        field(
            BT_146,
            &[NET_PRICE[0], NET_PRICE[1], ram("ChargeAmount")],
            Format::Amount,
        ),
        field(
            BT_149,
            &[NET_PRICE[0], NET_PRICE[1], ram("BasisQuantity")],
            Format::Quantity,
        ),
        field(
            BT_131,
            &[
                LINE_SETTLEMENT,
                ram("SpecifiedTradeSettlementLineMonetarySummation"),
                ram("LineTotalAmount"),
            ],
            Format::Amount,
        ),
        field(
            BT_151,
            &[
                LINE_SETTLEMENT,
                ram("ApplicableTradeTax"),
                ram("CategoryCode"),
            ],
            Format::VatCategory,
        ),
        field(
            BT_152,
            &[
                LINE_SETTLEMENT,
                ram("ApplicableTradeTax"),
                ram("RateApplicablePercent"),
            ],
            Format::Text,
        ),
        field(
            BT_128,
            &[
                LINE_SETTLEMENT,
                ram("AdditionalReferencedDocument"),
                ram("IssuerAssignedID"),
            ],
            Format::Text,
        ),
        field(
            BT_132,
            &[
                ram("SpecifiedLineTradeAgreement"),
                ram("BuyerOrderReferencedDocument"),
                ram("LineID"),
            ],
            Format::Text,
        ),
        field(
            BT_133,
            &[
                LINE_SETTLEMENT,
                ram("ReceivableSpecifiedTradeAccountingAccount"),
                ram("ID"),
            ],
            Format::Text,
        ),
        field(
            BT_134,
            &[LINE_SETTLEMENT, PERIOD, ram("StartDateTime"), DATE],
            Format::Date,
        ),
        field(
            BT_135,
            &[LINE_SETTLEMENT, PERIOD, ram("EndDateTime"), DATE],
            Format::Date,
        ),
    ],
}];

/// Sections after the line items.
const TRAILER_SECTIONS: &[SectionDef] = &[
    SectionDef {
        title: PAYMENT,
        anchor: &[SETTLEMENT[0], SETTLEMENT[1]],
        repeatable: false,
        fallback: None,
        title_suffix: None,
        fields: &[
            field(
                BT_81,
                &[ram("SpecifiedTradeSettlementPaymentMeans"), ram("TypeCode")],
                Format::PaymentMeans,
            ),
            field(BT_83, &[ram("PaymentReference")], Format::Text),
            field(
                BT_84,
                &[
                    ram("SpecifiedTradeSettlementPaymentMeans"),
                    ram("PayeePartyCreditorFinancialAccount"),
                    ram("IBANID"),
                ],
                Format::Iban,
            ),
            field(
                BT_85,
                &[
                    ram("SpecifiedTradeSettlementPaymentMeans"),
                    ram("PayeePartyCreditorFinancialAccount"),
                    ram("AccountName"),
                ],
                Format::Text,
            ),
            field(
                BT_86,
                &[
                    ram("SpecifiedTradeSettlementPaymentMeans"),
                    ram("PayeeSpecifiedCreditorFinancialInstitution"),
                    ram("BICID"),
                ],
                Format::Text,
            ),
            field(
                BT_89,
                &[
                    ram("SpecifiedTradePaymentTerms"),
                    ram("DirectDebitMandateID"),
                ],
                Format::Text,
            ),
            field(BT_90, &[ram("CreditorReferenceID")], Format::Text),
            field(
                BT_91,
                &[
                    ram("SpecifiedTradeSettlementPaymentMeans"),
                    ram("PayerPartyDebtorFinancialAccount"),
                    ram("IBANID"),
                ],
                Format::Iban,
            ),
            field(
                BT_20,
                &[ram("SpecifiedTradePaymentTerms"), ram("Description")],
                Format::Text,
            ),
        ],
    },
    SectionDef {
        title: ALLOWANCE,
        anchor: &[
            SETTLEMENT[0],
            SETTLEMENT[1],
            ALLOWANCE_CHARGE.with(Condition::ChildText(CHARGE_INDICATOR, "false")),
        ],
        fallback: None,
        repeatable: true,
        title_suffix: None,
        fields: &[
            field(BT_92, &[ram("ActualAmount")], Format::Amount),
            field(BT_93, &[ram("BasisAmount")], Format::Amount),
            field(BT_94, &[ram("CalculationPercent")], Format::Text),
            field(
                BT_95,
                &[ALLOWANCE_CATEGORY, ram("CategoryCode")],
                Format::VatCategory,
            ),
            field(
                BT_96,
                &[ALLOWANCE_CATEGORY, ram("RateApplicablePercent")],
                Format::Text,
            ),
            field(BT_97, &[ram("Reason")], Format::Text),
            field(BT_98, &[ram("ReasonCode")], Format::Text),
        ],
    },
    SectionDef {
        title: CHARGE,
        anchor: &[
            SETTLEMENT[0],
            SETTLEMENT[1],
            ALLOWANCE_CHARGE.with(Condition::ChildText(CHARGE_INDICATOR, "true")),
        ],
        fallback: None,
        repeatable: true,
        title_suffix: None,
        fields: &[
            field(BT_99, &[ram("ActualAmount")], Format::Amount),
            field(BT_100, &[ram("BasisAmount")], Format::Amount),
            field(BT_101, &[ram("CalculationPercent")], Format::Text),
            field(
                BT_102,
                &[ALLOWANCE_CATEGORY, ram("CategoryCode")],
                Format::VatCategory,
            ),
            field(
                BT_103,
                &[ALLOWANCE_CATEGORY, ram("RateApplicablePercent")],
                Format::Text,
            ),
            field(BT_104, &[ram("Reason")], Format::Text),
            field(BT_105, &[ram("ReasonCode")], Format::Text),
        ],
    },
    SectionDef {
        title: VAT_BREAKDOWN,
        anchor: &[SETTLEMENT[0], SETTLEMENT[1], ram("ApplicableTradeTax")],
        repeatable: true,
        fallback: None,
        title_suffix: None,
        fields: &[
            field(BT_116, &[ram("BasisAmount")], Format::Amount),
            field(BT_117, &[ram("CalculatedAmount")], Format::Amount),
            field(BT_118, &[ram("CategoryCode")], Format::VatCategory),
            field(BT_119, &[ram("RateApplicablePercent")], Format::Text),
            field(BT_120, &[ram("ExemptionReason")], Format::Text),
        ],
    },
    SectionDef {
        title: TOTALS,
        anchor: &[
            SETTLEMENT[0],
            SETTLEMENT[1],
            ram("SpecifiedTradeSettlementHeaderMonetarySummation"),
        ],
        repeatable: false,
        fallback: None,
        title_suffix: None,
        fields: &[
            field(BT_106, &[ram("LineTotalAmount")], Format::Amount),
            field(BT_107, &[ram("AllowanceTotalAmount")], Format::Amount),
            field(BT_108, &[ram("ChargeTotalAmount")], Format::Amount),
            field(BT_109, &[ram("TaxBasisTotalAmount")], Format::Amount),
            field(BT_110, &[ram("TaxTotalAmount")], Format::Amount),
            field(BT_112, &[ram("GrandTotalAmount")], Format::Amount),
            field(BT_113, &[ram("TotalPrepaidAmount")], Format::Amount),
            field(BT_114, &[ram("RoundingAmount")], Format::Amount),
            field(BT_115, &[ram("DuePayableAmount")], Format::Amount),
        ],
    },
];
