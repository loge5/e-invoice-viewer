//! Display rules for field values (spec 0004, "Display rules").

use crate::detect::{Detection, Profile};
use crate::i18n::{Language, n_, tr};

/// Formats a format-102 date (`YYYYMMDD`) as a date in `language`. Other
/// formats and invalid dates are returned unchanged.
pub fn date(raw: &str, format: Option<&str>, language: Language) -> String {
    let trimmed = raw.trim();
    let parts = (trimmed.len() == 8 && trimmed.is_ascii())
        .then(|| (&trimmed[0..4], &trimmed[4..6], &trimmed[6..8]));
    match parts {
        Some(parts) if format == Some("102") => valid_date(parts, language),
        _ => None,
    }
    .unwrap_or_else(|| raw.to_string())
}

/// A UBL date `YYYY-MM-DD` as a date in `language`; anything else unchanged.
pub fn iso_date(raw: &str, language: Language) -> String {
    match raw.trim().split('-').collect::<Vec<_>>()[..] {
        [year, month, day] => valid_date((year, month, day), language),
        _ => None,
    }
    .unwrap_or_else(|| raw.to_string())
}

/// A valid date given as four, two and two digits, written as `DD.MM.YYYY`
/// in German and `YYYY-MM-DD` in English.
fn valid_date((year, month, day): (&str, &str, &str), language: Language) -> Option<String> {
    let digits = |text: &str, len: usize| {
        (text.len() == len && text.bytes().all(|b| b.is_ascii_digit()))
            .then(|| text.parse::<u32>().ok())
            .flatten()
    };
    let (y, m, d) = (digits(year, 4)?, digits(month, 2)?, digits(day, 2)?);
    ((1..=12).contains(&m) && (1..=days_in_month(y, m)).contains(&d)).then(|| match language {
        Language::German => format!("{day}.{month}.{year}"),
        Language::English => format!("{year}-{month}-{day}"),
    })
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 if (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400) => {
            29
        }
        2 => 28,
        _ => 31,
    }
}

/// Formats a decimal amount with all decimal places but at least two,
/// followed by the currency. Values that are not plain decimal numbers are
/// returned unchanged.
pub fn amount(raw: &str, currency: Option<&str>, language: Language) -> String {
    let Some(number) = decimal(raw, 2, language) else {
        return raw.to_string();
    };
    match currency {
        Some(currency) if !currency.is_empty() => format!("{number} {currency}"),
        _ => number,
    }
}

/// The decimal and thousands separators of `language`.
fn separators(language: Language) -> (char, char) {
    match language {
        Language::German => (',', '.'),
        Language::English => ('.', ','),
    }
}

/// Formats a decimal number with the separators of `language`, padding the
/// decimal part to at least `min_decimals` places. Returns `None` if `raw` is
/// not a plain decimal number (optional sign, digits, optional `.` and
/// digits).
fn decimal(raw: &str, min_decimals: usize, language: Language) -> Option<String> {
    let text = raw.trim();
    let (sign, unsigned) = match text.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", text.strip_prefix('+').unwrap_or(text)),
    };
    let (integer, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    let is_digits = |part: &str| part.bytes().all(|b| b.is_ascii_digit());
    if integer.is_empty() || !is_digits(integer) || !is_digits(fraction) {
        return None;
    }
    let integer = integer.trim_start_matches('0');
    let integer = if integer.is_empty() { "0" } else { integer };

    let (decimal_separator, thousands_separator) = separators(language);
    let mut grouped = String::new();
    for (index, digit) in integer.chars().enumerate() {
        if index > 0 && (integer.len() - index).is_multiple_of(3) {
            grouped.push(thousands_separator);
        }
        grouped.push(digit);
    }
    let mut fraction = fraction.to_string();
    while fraction.len() < min_decimals {
        fraction.push('0');
    }
    Some(if fraction.is_empty() {
        format!("{sign}{grouped}")
    } else {
        format!("{sign}{grouped}{decimal_separator}{fraction}")
    })
}

/// Formats a quantity without trailing zeros, followed by the unit name and
/// code (`8 hours (HUR)`), or only the code for units without a known name
/// (`2.5 XPP`).
pub fn quantity(raw: &str, unit: Option<&str>, language: Language) -> String {
    let (decimal_separator, _) = separators(language);
    let number = match decimal(raw, 0, language) {
        Some(number) if number.contains(decimal_separator) => number
            .trim_end_matches('0')
            .trim_end_matches(decimal_separator)
            .to_string(),
        Some(number) => number,
        None => return raw.to_string(),
    };
    match unit.map(str::trim).filter(|unit| !unit.is_empty()) {
        Some(unit) => match UNIT_NAMES.iter().find(|(code, _)| *code == unit) {
            Some((code, name)) => format!("{number} {} ({code})", tr(language, name)),
            None => format!("{number} {unit}"),
        },
        None => number,
    }
}

/// Formats a code as `<name> (<code>)` in `language` if it is in `names`,
/// otherwise raw.
pub fn code(raw: &str, names: CodeNames, language: Language) -> String {
    names
        .iter()
        .find(|(code, _)| *code == raw.trim())
        .map(|(code, name)| format!("{} ({code})", tr(language, name)))
        .unwrap_or_else(|| raw.to_string())
}

/// Formats an IBAN in groups of four characters.
pub fn iban(raw: &str) -> String {
    let compact: Vec<char> = raw.chars().filter(|c| !c.is_whitespace()).collect();
    compact
        .chunks(4)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Formats an electronic address with its scheme (BT-34, BT-49), e.g.
/// `04011000-12345-34 (0204)`; without a scheme, the address alone.
pub fn electronic_address(raw: &str, scheme: Option<&str>) -> String {
    match scheme.map(str::trim).filter(|scheme| !scheme.is_empty()) {
        Some(scheme) => format!("{} ({scheme})", raw.trim()),
        None => raw.trim().to_string(),
    }
}

/// Formats the specification identifier (BT-24) as the detected profile name.
pub fn specification_identifier(raw: &str, detection: &Detection) -> String {
    let version = detection.profile_version.as_deref().unwrap_or_default();
    match detection.profile {
        Profile::Minimum => "Minimum".to_string(),
        Profile::BasicWl => "Basic WL".to_string(),
        Profile::Basic => "Basic".to_string(),
        Profile::En16931 => "EN 16931".to_string(),
        Profile::Extended => "Extended".to_string(),
        Profile::XRechnung if detection.extension => format!("XRechnung {version} (Extension)"),
        Profile::XRechnung => format!("XRechnung {version}"),
        Profile::Unknown => raw.to_string(),
    }
}

/// Codes with their English name, marked for translation.
pub type CodeNames = &'static [(&'static str, &'static str)];

/// Invoice type codes (BT-3, UNTDID 1001).
pub const INVOICE_TYPE_CODES: CodeNames = &[
    ("326", n_("Partial invoice")),
    ("380", n_("Commercial invoice")),
    ("381", n_("Credit note")),
    ("384", n_("Corrected invoice")),
    ("389", n_("Self-billed invoice")),
    ("875", n_("Partial construction invoice")),
    ("876", n_("Partial final construction invoice")),
    ("877", n_("Final construction invoice")),
];

/// Payment means codes (BT-81, UNTDID 4461).
pub const PAYMENT_MEANS_CODES: CodeNames = &[
    ("1", n_("Instrument not defined")),
    ("10", n_("In cash")),
    ("20", n_("Cheque")),
    ("30", n_("Credit transfer")),
    ("42", n_("Payment to bank account")),
    ("48", n_("Bank card")),
    ("49", n_("Direct debit")),
    ("54", n_("Credit card")),
    ("57", n_("Standing agreement")),
    ("58", n_("SEPA credit transfer")),
    ("59", n_("SEPA direct debit")),
    ("97", n_("Clearing between partners")),
];

/// VAT category codes (BT-118, BT-151, UNTDID 5305).
pub const VAT_CATEGORY_CODES: CodeNames = &[
    ("S", n_("Standard rate")),
    ("Z", n_("Zero rated goods")),
    ("E", n_("Exempt from tax")),
    ("AE", n_("VAT reverse charge")),
    ("K", n_("Intra-community supply")),
    ("G", n_("Export outside the EU")),
    ("O", n_("Outside the scope of VAT")),
    ("L", n_("Canary Islands IGIC")),
    ("M", n_("Ceuta and Melilla IPSI")),
];

/// Unit codes (BT-130, UN/ECE Recommendation 20) with a name, in the plural.
pub const UNIT_NAMES: CodeNames = &[
    ("C62", n_("pieces")),
    ("H87", n_("pieces")),
    ("HUR", n_("hours")),
    ("DAY", n_("days")),
    ("MON", n_("months")),
    ("KGM", n_("kilograms")),
    ("MTR", n_("metres")),
    ("LTR", n_("litres")),
    ("LS", n_("lump sum")),
];

#[cfg(test)]
mod tests {
    use super::*;

    const DE: Language = Language::German;
    const EN: Language = Language::English;

    // Spec 0017 AC3
    #[test]
    fn english_code_and_unit_names() {
        let table: [(CodeNames, &str, &str); 29] = [
            (INVOICE_TYPE_CODES, "326", "Partial invoice"),
            (INVOICE_TYPE_CODES, "380", "Commercial invoice"),
            (INVOICE_TYPE_CODES, "381", "Credit note"),
            (INVOICE_TYPE_CODES, "384", "Corrected invoice"),
            (INVOICE_TYPE_CODES, "389", "Self-billed invoice"),
            (INVOICE_TYPE_CODES, "875", "Partial construction invoice"),
            (
                INVOICE_TYPE_CODES,
                "876",
                "Partial final construction invoice",
            ),
            (INVOICE_TYPE_CODES, "877", "Final construction invoice"),
            (PAYMENT_MEANS_CODES, "1", "Instrument not defined"),
            (PAYMENT_MEANS_CODES, "10", "In cash"),
            (PAYMENT_MEANS_CODES, "20", "Cheque"),
            (PAYMENT_MEANS_CODES, "30", "Credit transfer"),
            (PAYMENT_MEANS_CODES, "42", "Payment to bank account"),
            (PAYMENT_MEANS_CODES, "48", "Bank card"),
            (PAYMENT_MEANS_CODES, "49", "Direct debit"),
            (PAYMENT_MEANS_CODES, "54", "Credit card"),
            (PAYMENT_MEANS_CODES, "57", "Standing agreement"),
            (PAYMENT_MEANS_CODES, "58", "SEPA credit transfer"),
            (PAYMENT_MEANS_CODES, "59", "SEPA direct debit"),
            (PAYMENT_MEANS_CODES, "97", "Clearing between partners"),
            (VAT_CATEGORY_CODES, "S", "Standard rate"),
            (VAT_CATEGORY_CODES, "Z", "Zero rated goods"),
            (VAT_CATEGORY_CODES, "E", "Exempt from tax"),
            (VAT_CATEGORY_CODES, "AE", "VAT reverse charge"),
            (VAT_CATEGORY_CODES, "K", "Intra-community supply"),
            (VAT_CATEGORY_CODES, "G", "Export outside the EU"),
            (VAT_CATEGORY_CODES, "O", "Outside the scope of VAT"),
            (VAT_CATEGORY_CODES, "L", "Canary Islands IGIC"),
            (VAT_CATEGORY_CODES, "M", "Ceuta and Melilla IPSI"),
        ];
        for (names, raw, name) in table {
            assert_eq!(code(raw, names, EN), format!("{name} ({raw})"));
        }
        assert_eq!(code("381", INVOICE_TYPE_CODES, DE), "Gutschrift (381)");
        for (unit, name) in [
            ("C62", "pieces"),
            ("H87", "pieces"),
            ("HUR", "hours"),
            ("DAY", "days"),
            ("MON", "months"),
            ("KGM", "kilograms"),
            ("MTR", "metres"),
            ("LTR", "litres"),
            ("LS", "lump sum"),
        ] {
            assert_eq!(quantity("2", Some(unit), EN), format!("2 {name} ({unit})"));
        }
        assert_eq!(quantity("2", Some("C62"), DE), "2 Stück (C62)");
        assert_eq!(quantity("2.5", Some("XPP"), EN), "2.5 XPP");
        assert_eq!(quantity("2.5", Some("XPP"), DE), "2,5 XPP");
    }

    // Spec 0017 AC4
    #[test]
    fn number_formats_per_language() {
        assert_eq!(amount("1234567.5", Some("EUR"), DE), "1.234.567,50 EUR");
        assert_eq!(amount("1234567.5", Some("EUR"), EN), "1,234,567.50 EUR");
        assert_eq!(quantity("2.50", None, DE), "2,5");
        assert_eq!(quantity("2.50", None, EN), "2.5");
        assert_eq!(amount("-30.00", Some("EUR"), DE), "-30,00 EUR");
        assert_eq!(amount("-30.00", Some("EUR"), EN), "-30.00 EUR");
        for language in Language::ALL {
            assert_eq!(amount("12,50", Some("EUR"), language), "12,50");
        }
    }

    // Spec 0017 AC5
    #[test]
    fn dates_per_language() {
        assert_eq!(date("20261117", Some("102"), DE), "17.11.2026");
        assert_eq!(date("20261117", Some("102"), EN), "2026-11-17");
        assert_eq!(iso_date("2026-11-17", DE), "17.11.2026");
        assert_eq!(iso_date("2026-11-17", EN), "2026-11-17");
        for language in Language::ALL {
            assert_eq!(date("20261332", Some("102"), language), "20261332");
        }
    }
}
