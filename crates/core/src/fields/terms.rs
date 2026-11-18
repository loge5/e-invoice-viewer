//! The business terms shown as fields, the same for CII and UBL: number and
//! label. Labels are the official EN 16931 names (spec 0017), translated
//! where they are shown.

use super::{Term, term};
use crate::i18n::n_;

pub(super) const BT_1: Term = term("BT-1", "Invoice number");
pub(super) const BT_2: Term = term("BT-2", "Invoice issue date");
pub(super) const BT_3: Term = term("BT-3", "Invoice type code");
pub(super) const BT_5: Term = term("BT-5", "Invoice currency code");
pub(super) const BT_9: Term = term("BT-9", "Payment due date");
pub(super) const BT_10: Term = term("BT-10", "Buyer reference");
pub(super) const BT_20: Term = term("BT-20", "Payment terms");
pub(super) const BT_22: Term = term("BT-22", "Invoice note");
pub(super) const BT_24: Term = term("BT-24", "Specification identifier");
pub(super) const BT_27: Term = term("BT-27", "Seller name");
pub(super) const BT_29: Term = term("BT-29", "Seller identifier");
pub(super) const BT_31: Term = term("BT-31", "Seller VAT identifier");
pub(super) const BT_32: Term = term("BT-32", "Seller tax registration identifier");
pub(super) const BT_35: Term = term("BT-35", "Seller address line 1");
pub(super) const BT_36: Term = term("BT-36", "Seller address line 2");
pub(super) const BT_37: Term = term("BT-37", "Seller city");
pub(super) const BT_38: Term = term("BT-38", "Seller post code");
pub(super) const BT_40: Term = term("BT-40", "Seller country code");
pub(super) const BT_41: Term = term("BT-41", "Seller contact point");
pub(super) const BT_42: Term = term("BT-42", "Seller contact telephone number");
pub(super) const BT_43: Term = term("BT-43", "Seller contact email address");
pub(super) const BT_44: Term = term("BT-44", "Buyer name");
pub(super) const BT_46: Term = term("BT-46", "Buyer identifier");
pub(super) const BT_48: Term = term("BT-48", "Buyer VAT identifier");
pub(super) const BT_50: Term = term("BT-50", "Buyer address line 1");
pub(super) const BT_51: Term = term("BT-51", "Buyer address line 2");
pub(super) const BT_52: Term = term("BT-52", "Buyer city");
pub(super) const BT_53: Term = term("BT-53", "Buyer post code");
pub(super) const BT_55: Term = term("BT-55", "Buyer country code");
pub(super) const BT_81: Term = term("BT-81", "Payment means type code");
pub(super) const BT_83: Term = term("BT-83", "Remittance information");
pub(super) const BT_84: Term = term("BT-84", "Payment account identifier");
pub(super) const BT_85: Term = term("BT-85", "Payment account name");
pub(super) const BT_86: Term = term("BT-86", "Payment service provider identifier");
pub(super) const BT_106: Term = term("BT-106", "Sum of Invoice line net amount");
pub(super) const BT_107: Term = term("BT-107", "Sum of allowances on document level");
pub(super) const BT_108: Term = term("BT-108", "Sum of charges on document level");
pub(super) const BT_109: Term = term("BT-109", "Invoice total amount without VAT");
pub(super) const BT_110: Term = term("BT-110", "Invoice total VAT amount");
pub(super) const BT_112: Term = term("BT-112", "Invoice total amount with VAT");
pub(super) const BT_113: Term = term("BT-113", "Paid amount");
pub(super) const BT_114: Term = term("BT-114", "Rounding amount");
pub(super) const BT_115: Term = term("BT-115", "Amount due for payment");
pub(super) const BT_116: Term = term("BT-116", "VAT category taxable amount");
pub(super) const BT_117: Term = term("BT-117", "VAT category tax amount");
pub(super) const BT_118: Term = term("BT-118", "VAT category code");
pub(super) const BT_119: Term = term("BT-119", "VAT category rate");
pub(super) const BT_120: Term = term("BT-120", "VAT exemption reason text");
pub(super) const BT_126: Term = term("BT-126", "Invoice line identifier");
pub(super) const BT_127: Term = term("BT-127", "Invoice line note");
pub(super) const BT_128: Term = term("BT-128", "Invoice line object identifier");
pub(super) const BT_129: Term = term("BT-129", "Invoiced quantity");
pub(super) const BT_130: Term = term("BT-130", "Invoiced quantity unit of measure code");
pub(super) const BT_131: Term = term("BT-131", "Invoice line net amount");
pub(super) const BT_133: Term = term("BT-133", "Invoice line Buyer accounting reference");
pub(super) const BT_146: Term = term("BT-146", "Item net price");
pub(super) const BT_149: Term = term("BT-149", "Item price base quantity");
pub(super) const BT_151: Term = term("BT-151", "Invoiced item VAT category code");
pub(super) const BT_152: Term = term("BT-152", "Invoiced item VAT rate");
pub(super) const BT_153: Term = term("BT-153", "Item name");
pub(super) const BT_154: Term = term("BT-154", "Item description");
pub(super) const BT_155: Term = term("BT-155", "Item Seller's identifier");

// Section titles.
pub(super) const INVOICE: &str = n_("Invoice");
pub(super) const SELLER: &str = n_("Seller (BG-4)");
pub(super) const BUYER: &str = n_("Buyer (BG-7)");
pub(super) const LINE: &str = n_("Invoice line");
pub(super) const PAYMENT: &str = n_("Payment instructions (BG-16)");
pub(super) const VAT_BREAKDOWN: &str = n_("VAT breakdown (BG-23)");
pub(super) const TOTALS: &str = n_("Document totals (BG-22)");
