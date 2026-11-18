//! Display rules for field values (spec 0004, "Display rules").

use crate::detect::{Detection, Profile};

/// Formats a format-102 date (`YYYYMMDD`) as `DD.MM.YYYY`. Other formats and
/// invalid dates are returned unchanged.
pub fn date(raw: &str, format: Option<&str>) -> String {
    let raw_trimmed = raw.trim();
    if format != Some("102")
        || raw_trimmed.len() != 8
        || !raw_trimmed.bytes().all(|b| b.is_ascii_digit())
    {
        return raw.to_string();
    }
    let (year, month, day) = (&raw_trimmed[0..4], &raw_trimmed[4..6], &raw_trimmed[6..8]);
    let (Ok(y), Ok(m), Ok(d)) = (
        year.parse::<u32>(),
        month.parse::<u32>(),
        day.parse::<u32>(),
    ) else {
        return raw.to_string();
    };
    if m == 0 || m > 12 || d == 0 || d > days_in_month(y, m) {
        return raw.to_string();
    }
    format!("{day}.{month}.{year}")
}

/// A UBL date `YYYY-MM-DD` as `DD.MM.YYYY`; anything else unchanged.
pub fn iso_date(raw: &str) -> String {
    let trimmed = raw.trim();
    let parts: Vec<&str> = trimmed.split('-').collect();
    match parts[..] {
        [year, month, day] if year.len() == 4 && month.len() == 2 && day.len() == 2 => {
            let compact = format!("{year}{month}{day}");
            let formatted = date(&compact, Some("102"));
            if formatted == compact {
                raw.to_string()
            } else {
                formatted
            }
        }
        _ => raw.to_string(),
    }
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

/// Formats a decimal amount with German separators, all decimal places but at
/// least two, followed by the currency. Values that are not plain decimal
/// numbers are returned unchanged.
pub fn amount(raw: &str, currency: Option<&str>) -> String {
    let Some(number) = decimal(raw, 2) else {
        return raw.to_string();
    };
    match currency {
        Some(currency) if !currency.is_empty() => format!("{number} {currency}"),
        _ => number,
    }
}

/// Formats a decimal number with German separators, padding the decimal part
/// to at least `min_decimals` places. Returns `None` if `raw` is not a plain
/// decimal number (optional sign, digits, optional `.` and digits).
pub fn decimal(raw: &str, min_decimals: usize) -> Option<String> {
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

    let mut grouped = String::new();
    for (index, digit) in integer.chars().enumerate() {
        if index > 0 && (integer.len() - index) % 3 == 0 {
            grouped.push('.');
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
        format!("{sign}{grouped},{fraction}")
    })
}

/// Formats a quantity with German separators and without trailing zeros,
/// followed by the unit name and code (`8 Stunden (HUR)`), or only the code
/// for units without a known name (`2,5 XPP`).
pub fn quantity(raw: &str, unit: Option<&str>) -> String {
    let number = match decimal(raw, 0) {
        Some(number) if number.contains(',') => number
            .trim_end_matches('0')
            .trim_end_matches(',')
            .to_string(),
        Some(number) => number,
        None => return raw.to_string(),
    };
    match unit.map(str::trim).filter(|unit| !unit.is_empty()) {
        Some(unit) => match UNIT_NAMES.iter().find(|(code, _)| *code == unit) {
            Some((code, name)) => format!("{number} {name} ({code})"),
            None => format!("{number} {unit}"),
        },
        None => number,
    }
}

/// Formats a code as `<name> (<code>)` if it is in `names`, otherwise raw.
pub fn code(raw: &str, names: &[(&str, &str)]) -> String {
    names
        .iter()
        .find(|(code, _)| *code == raw.trim())
        .map(|(code, name)| format!("{name} ({code})"))
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

/// Invoice type codes (BT-3, UNTDID 1001).
pub const INVOICE_TYPE_CODES: &[(&str, &str)] = &[
    ("326", "Teilrechnung"),
    ("380", "Rechnung"),
    ("381", "Gutschrift"),
    ("384", "Korrekturrechnung"),
    ("389", "Selbst ausgestellte Rechnung"),
    ("875", "Abschlagsrechnung (Bau)"),
    ("876", "Teilschlussrechnung (Bau)"),
    ("877", "Schlussrechnung (Bau)"),
];

/// Payment means codes (BT-81, UNTDID 4461).
pub const PAYMENT_MEANS_CODES: &[(&str, &str)] = &[
    ("1", "Nicht definiert"),
    ("10", "Barzahlung"),
    ("20", "Scheck"),
    ("30", "Überweisung"),
    ("42", "Zahlung auf Bankkonto"),
    ("48", "Bankkarte"),
    ("49", "Lastschrift"),
    ("54", "Kreditkarte"),
    ("57", "Dauerauftrag"),
    ("58", "SEPA-Überweisung"),
    ("59", "SEPA-Lastschrift"),
    ("97", "Verrechnung"),
];

/// VAT category codes (BT-118, BT-151, UNTDID 5305).
pub const VAT_CATEGORY_CODES: &[(&str, &str)] = &[
    ("S", "Normalsatz"),
    ("Z", "Nullsatz"),
    ("E", "Steuerbefreit"),
    ("AE", "Steuerschuldnerschaft des Leistungsempfängers"),
    ("K", "Innergemeinschaftliche Lieferung"),
    ("G", "Ausfuhr außerhalb der EU"),
    ("O", "Nicht steuerbar"),
    ("L", "Kanarische Inseln (IGIC)"),
    ("M", "Ceuta und Melilla (IPSI)"),
];

/// Unit codes (BT-130, UN/ECE Recommendation 20) with a German name.
pub const UNIT_NAMES: &[(&str, &str)] = &[
    ("C62", "Stück"),
    ("H87", "Stück"),
    ("HUR", "Stunden"),
    ("DAY", "Tage"),
    ("MON", "Monate"),
    ("KGM", "Kilogramm"),
    ("MTR", "Meter"),
    ("LTR", "Liter"),
    ("LS", "pauschal"),
];
