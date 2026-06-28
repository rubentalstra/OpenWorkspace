//! Minimal phone-number model: stores raw national digits and formats them for
//! display per country. Not a full libphonenumber — it groups digits with a
//! country template and caps the digit count; validation is left to the caller.

use crate::utils::country::Country;

/// A phone number held as raw digits (no dial code, no separators).
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct PhoneNumber {
    digits: String,
}

impl PhoneNumber {
    /// Builds a number from arbitrary input, keeping only the first `max_digits`
    /// ASCII digits.
    pub fn new(input: &str, max_digits: usize) -> Self {
        let digits = input
            .chars()
            .filter(char::is_ascii_digit)
            .take(max_digits)
            .collect();
        Self { digits }
    }

    /// Whether no digits have been entered.
    pub fn is_empty(&self) -> bool {
        self.digits.is_empty()
    }

    /// The raw digits, without grouping or dial code.
    pub fn digits(&self) -> &str {
        &self.digits
    }

    /// The digits grouped for display using `country`'s national format.
    pub fn format(&self, country: Country) -> String {
        PhoneFormat::for_country(country).group(&self.digits)
    }
}

/// National display format for a country's phone numbers.
#[derive(Clone, Copy)]
pub struct PhoneFormat {
    /// Maximum national-significant digits accepted.
    pub max_digits: usize,
    /// Display template: `#` is a digit slot, every other character is a literal
    /// separator emitted only while digits remain.
    template: &'static str,
    placeholder: &'static str,
}

impl PhoneFormat {
    /// The display format for `country`, falling back to an ungrouped 15-digit
    /// E.164 maximum for countries without a specific pattern.
    pub fn for_country(country: Country) -> Self {
        match country {
            Country::UnitedStatesOfAmerica | Country::Canada => Self {
                max_digits: 10,
                template: "(###) ###-####",
                placeholder: "(201) 555-0123",
            },
            Country::UnitedKingdom => Self {
                max_digits: 10,
                template: "##### ######",
                placeholder: "07400 123456",
            },
            Country::France => Self {
                max_digits: 9,
                template: "# ## ## ## ##",
                placeholder: "6 12 34 56 78",
            },
            Country::Germany => Self {
                max_digits: 11,
                template: "#### #######",
                placeholder: "1512 3456789",
            },
            Country::Spain | Country::Italy => Self {
                max_digits: 9,
                template: "### ## ## ##",
                placeholder: "612 34 56 78",
            },
            Country::Australia => Self {
                max_digits: 9,
                template: "### ### ###",
                placeholder: "412 345 678",
            },
            Country::India => Self {
                max_digits: 10,
                template: "##### #####",
                placeholder: "98765 43210",
            },
            Country::Brazil => Self {
                max_digits: 11,
                template: "(##) #####-####",
                placeholder: "(11) 91234-5678",
            },
            _ => Self {
                max_digits: 15,
                template: "",
                placeholder: "Phone number",
            },
        }
    }

    /// The example shown when the field is empty.
    pub fn placeholder(self) -> &'static str {
        self.placeholder
    }

    /// Groups `digits` per this format, emitting a separator only while more
    /// digits remain (so partial input formats cleanly). An empty template
    /// returns the digits unchanged.
    fn group(self, digits: &str) -> String {
        let mut chars = digits.chars().peekable();
        let mut out = String::new();
        for slot in self.template.chars() {
            if chars.peek().is_none() {
                break;
            }
            if slot == '#' {
                if let Some(digit) = chars.next() {
                    out.push(digit);
                }
            } else {
                out.push(slot);
            }
        }
        out.extend(chars);
        out
    }
}
