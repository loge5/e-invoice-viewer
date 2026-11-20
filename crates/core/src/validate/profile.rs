//! The ZUGFeRD/Factur-X profile schemas and rules (spec 0027), bundled
//! unchanged from the FeRD package (`rules/facturx-1.09.2/SOURCE.md`).

use crate::detect::{Profile, Syntax};

/// Version of the bundled Factur-X artefacts, and the ZUGFeRD version they
/// belong to.
pub(super) const VERSION: &str = "1.09.2";
pub(super) const ZUGFERD_VERSION: &str = "2.5.2";

/// Rule ID of a profile rule that has no `id` in the Schematron.
pub(super) const UNNAMED_RULE_ID: &str = "FX";

/// A Factur-X profile whose schema and rules an invoice is checked against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum FacturX {
    Minimum,
    BasicWl,
    Basic,
    En16931,
    Extended,
}

/// The bundled files of one profile.
struct Files {
    name: &'static str,
    /// Address of the main XSD below the schema scheme of `xsd`.
    main_schema: &'static str,
    /// The XSD files by their address below the schema scheme.
    schemas: [(&'static str, &'static str); 4],
    schematron: &'static str,
    /// The code list file, by the name the Schematron reads it with.
    code_lists: (&'static str, &'static str),
}

macro_rules! profile_files {
    ($name:literal, $dir:literal, $xsd:literal, $sch:literal) => {
        Files {
            name: $name,
            main_schema: concat!("facturx/", $dir, "/Factur-X_1.09.2_", $xsd, ".xsd"),
            schemas: [
                profile_files!(@xsd $dir, $xsd, ""),
                profile_files!(@xsd $dir, $xsd, "_urn_un_unece_uncefact_data_standard_QualifiedDataType_100"),
                profile_files!(@xsd $dir, $xsd, "_urn_un_unece_uncefact_data_standard_ReusableAggregateBusinessInformationEntity_100"),
                profile_files!(@xsd $dir, $xsd, "_urn_un_unece_uncefact_data_standard_UnqualifiedDataType_100"),
            ],
            schematron: include_str!(concat!(
                "../../rules/facturx-1.09.2/", $dir, "/FACTUR-X_", $sch, ".sch"
            )),
            code_lists: (
                concat!("FACTUR-X_", $sch, "_codedb.xml"),
                include_str!(concat!(
                    "../../rules/facturx-1.09.2/", $dir, "/FACTUR-X_", $sch, "_codedb.xml"
                )),
            ),
        }
    };
    (@xsd $dir:literal, $xsd:literal, $suffix:literal) => {
        (
            concat!("facturx/", $dir, "/Factur-X_1.09.2_", $xsd, $suffix, ".xsd"),
            include_str!(concat!(
                "../../rules/facturx-1.09.2/", $dir, "/Factur-X_1.09.2_", $xsd, $suffix, ".xsd"
            )),
        )
    };
}

const MINIMUM: Files = profile_files!("Minimum", "minimum", "MINIMUM", "MINIMUM");
const BASIC_WL: Files = profile_files!("Basic WL", "basic-wl", "BASICWL", "BASIC-WL");
const BASIC: Files = profile_files!("Basic", "basic", "BASIC", "BASIC");
const EN16931: Files = profile_files!("EN 16931", "en16931", "EN16931", "EN16931");
const EXTENDED: Files = profile_files!("Extended", "extended", "EXTENDED", "EXTENDED");

impl FacturX {
    pub(super) const ALL: [Self; 5] = [
        Self::Minimum,
        Self::BasicWl,
        Self::Basic,
        Self::En16931,
        Self::Extended,
    ];

    /// The profile rules that apply to an invoice: CII with a Factur-X
    /// profile, or with the plain EN 16931 identifier when embedded in a
    /// PDF. XRechnung, UBL and other invoices get none.
    pub(super) fn of(syntax: Syntax, profile: Profile, embedded: bool) -> Option<Self> {
        if syntax != Syntax::Cii {
            return None;
        }
        match profile {
            Profile::Minimum => Some(Self::Minimum),
            Profile::BasicWl => Some(Self::BasicWl),
            Profile::Basic => Some(Self::Basic),
            Profile::En16931 if embedded => Some(Self::En16931),
            Profile::Extended => Some(Self::Extended),
            Profile::En16931 | Profile::XRechnung | Profile::Unknown => None,
        }
    }

    fn files(self) -> &'static Files {
        match self {
            Self::Minimum => &MINIMUM,
            Self::BasicWl => &BASIC_WL,
            Self::Basic => &BASIC,
            Self::En16931 => &EN16931,
            Self::Extended => &EXTENDED,
        }
    }

    /// Index in [`Self::ALL`].
    pub(super) fn index(self) -> usize {
        self as usize
    }

    /// The name of the profile schema in the summary, e.g.
    /// `Factur-X 1.09.2 EN 16931`.
    pub(super) fn schema_name(self) -> String {
        format!("Factur-X {VERSION} {}", self.files().name)
    }

    /// The name of the profile rules in the summary, e.g.
    /// `Factur-X 1.09.2 (ZUGFeRD 2.5.2) EN 16931`.
    pub(super) fn rules_name(self) -> String {
        format!(
            "Factur-X {VERSION} (ZUGFeRD {ZUGFERD_VERSION}) {}",
            self.files().name
        )
    }

    /// Whether the EN 16931 rules apply too. Minimum and Basic WL are not
    /// complete invoices (spec 0009); Extended allows what EN 16931 forbids
    /// (e.g. sub-invoice lines), and its Schematron contains the EN 16931
    /// rules adapted to it, as in FeRD's own validation.
    pub(super) fn with_en16931_rules(self) -> bool {
        matches!(self, Self::Basic | Self::En16931)
    }

    pub(super) fn main_schema(self) -> &'static str {
        self.files().main_schema
    }

    pub(super) fn schematron(self) -> &'static str {
        self.files().schematron
    }

    /// The code list file, by the name the Schematron reads it with.
    pub(super) fn code_lists(self) -> (&'static str, &'static str) {
        self.files().code_lists
    }
}

/// The bundled XSD file at an address below the schema scheme of `xsd`.
pub(super) fn schema_file(address: &str) -> Option<&'static str> {
    FacturX::ALL
        .iter()
        .flat_map(|profile| &profile.files().schemas)
        .find(|(file, _)| *file == address)
        .map(|(_, text)| *text)
}
