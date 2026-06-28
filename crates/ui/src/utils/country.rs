//! Country list with ISO-3166 alpha-2 codes, E.164 dial codes and flag emoji,
//! used by the phone-number input. A curated set of widely-used countries;
//! extend [`Country`] and its `data` match to add more.

/// A country selectable in the phone input.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Country {
    UnitedStatesOfAmerica,
    Canada,
    UnitedKingdom,
    Ireland,
    France,
    Germany,
    Spain,
    Italy,
    Portugal,
    Netherlands,
    Belgium,
    Luxembourg,
    Switzerland,
    Austria,
    Denmark,
    Norway,
    Sweden,
    Finland,
    Iceland,
    Poland,
    CzechRepublic,
    Slovakia,
    Hungary,
    Romania,
    Greece,
    Turkey,
    Russia,
    Ukraine,
    Australia,
    NewZealand,
    Japan,
    China,
    SouthKorea,
    India,
    Indonesia,
    Singapore,
    Malaysia,
    Thailand,
    Vietnam,
    Philippines,
    UnitedArabEmirates,
    SaudiArabia,
    Israel,
    SouthAfrica,
    Nigeria,
    Kenya,
    Egypt,
    Morocco,
    Brazil,
    Argentina,
    Chile,
    Colombia,
    Mexico,
}

/// Static descriptor for a [`Country`].
struct CountryData {
    name: &'static str,
    alpha2: &'static str,
    dial_code: &'static str,
    flag: &'static str,
}

impl Country {
    /// Every supported country, in display order (common ones can be filtered
    /// out by the caller).
    pub fn all() -> &'static [Country] {
        use Country::{
            Argentina, Australia, Austria, Belgium, Brazil, Canada, Chile, China, Colombia,
            CzechRepublic, Denmark, Egypt, Finland, France, Germany, Greece, Hungary, Iceland,
            India, Indonesia, Ireland, Israel, Italy, Japan, Kenya, Luxembourg, Malaysia, Mexico,
            Morocco, Netherlands, NewZealand, Nigeria, Norway, Philippines, Poland, Portugal,
            Romania, Russia, SaudiArabia, Singapore, Slovakia, SouthAfrica, SouthKorea, Spain,
            Sweden, Switzerland, Thailand, Turkey, Ukraine, UnitedArabEmirates, UnitedKingdom,
            UnitedStatesOfAmerica, Vietnam,
        };
        &[
            UnitedStatesOfAmerica,
            Canada,
            UnitedKingdom,
            Ireland,
            France,
            Germany,
            Spain,
            Italy,
            Portugal,
            Netherlands,
            Belgium,
            Luxembourg,
            Switzerland,
            Austria,
            Denmark,
            Norway,
            Sweden,
            Finland,
            Iceland,
            Poland,
            CzechRepublic,
            Slovakia,
            Hungary,
            Romania,
            Greece,
            Turkey,
            Russia,
            Ukraine,
            Australia,
            NewZealand,
            Japan,
            China,
            SouthKorea,
            India,
            Indonesia,
            Singapore,
            Malaysia,
            Thailand,
            Vietnam,
            Philippines,
            UnitedArabEmirates,
            SaudiArabia,
            Israel,
            SouthAfrica,
            Nigeria,
            Kenya,
            Egypt,
            Morocco,
            Brazil,
            Argentina,
            Chile,
            Colombia,
            Mexico,
        ]
    }

    /// The country's display name, e.g. `"United States"`.
    pub fn name(self) -> &'static str {
        self.data().name
    }

    /// The ISO-3166 alpha-2 code, e.g. `"US"`.
    pub fn alpha2(self) -> &'static str {
        self.data().alpha2
    }

    /// The flag emoji, e.g. `"🇺🇸"`.
    pub fn flag_emoji(self) -> &'static str {
        self.data().flag
    }

    /// The E.164 dial code without the leading `+`, e.g. `"1"`.
    pub fn dial_code(self) -> &'static str {
        self.data().dial_code
    }

    /// The dial code prefixed with `+`, e.g. `"+1"`.
    pub fn dial_code_formatted(self) -> String {
        format!("+{}", self.dial_code())
    }

    fn data(self) -> CountryData {
        let (name, alpha2, dial_code, flag) = match self {
            Self::UnitedStatesOfAmerica => ("United States", "US", "1", "🇺🇸"),
            Self::Canada => ("Canada", "CA", "1", "🇨🇦"),
            Self::UnitedKingdom => ("United Kingdom", "GB", "44", "🇬🇧"),
            Self::Ireland => ("Ireland", "IE", "353", "🇮🇪"),
            Self::France => ("France", "FR", "33", "🇫🇷"),
            Self::Germany => ("Germany", "DE", "49", "🇩🇪"),
            Self::Spain => ("Spain", "ES", "34", "🇪🇸"),
            Self::Italy => ("Italy", "IT", "39", "🇮🇹"),
            Self::Portugal => ("Portugal", "PT", "351", "🇵🇹"),
            Self::Netherlands => ("Netherlands", "NL", "31", "🇳🇱"),
            Self::Belgium => ("Belgium", "BE", "32", "🇧🇪"),
            Self::Luxembourg => ("Luxembourg", "LU", "352", "🇱🇺"),
            Self::Switzerland => ("Switzerland", "CH", "41", "🇨🇭"),
            Self::Austria => ("Austria", "AT", "43", "🇦🇹"),
            Self::Denmark => ("Denmark", "DK", "45", "🇩🇰"),
            Self::Norway => ("Norway", "NO", "47", "🇳🇴"),
            Self::Sweden => ("Sweden", "SE", "46", "🇸🇪"),
            Self::Finland => ("Finland", "FI", "358", "🇫🇮"),
            Self::Iceland => ("Iceland", "IS", "354", "🇮🇸"),
            Self::Poland => ("Poland", "PL", "48", "🇵🇱"),
            Self::CzechRepublic => ("Czech Republic", "CZ", "420", "🇨🇿"),
            Self::Slovakia => ("Slovakia", "SK", "421", "🇸🇰"),
            Self::Hungary => ("Hungary", "HU", "36", "🇭🇺"),
            Self::Romania => ("Romania", "RO", "40", "🇷🇴"),
            Self::Greece => ("Greece", "GR", "30", "🇬🇷"),
            Self::Turkey => ("Turkey", "TR", "90", "🇹🇷"),
            Self::Russia => ("Russia", "RU", "7", "🇷🇺"),
            Self::Ukraine => ("Ukraine", "UA", "380", "🇺🇦"),
            Self::Australia => ("Australia", "AU", "61", "🇦🇺"),
            Self::NewZealand => ("New Zealand", "NZ", "64", "🇳🇿"),
            Self::Japan => ("Japan", "JP", "81", "🇯🇵"),
            Self::China => ("China", "CN", "86", "🇨🇳"),
            Self::SouthKorea => ("South Korea", "KR", "82", "🇰🇷"),
            Self::India => ("India", "IN", "91", "🇮🇳"),
            Self::Indonesia => ("Indonesia", "ID", "62", "🇮🇩"),
            Self::Singapore => ("Singapore", "SG", "65", "🇸🇬"),
            Self::Malaysia => ("Malaysia", "MY", "60", "🇲🇾"),
            Self::Thailand => ("Thailand", "TH", "66", "🇹🇭"),
            Self::Vietnam => ("Vietnam", "VN", "84", "🇻🇳"),
            Self::Philippines => ("Philippines", "PH", "63", "🇵🇭"),
            Self::UnitedArabEmirates => ("United Arab Emirates", "AE", "971", "🇦🇪"),
            Self::SaudiArabia => ("Saudi Arabia", "SA", "966", "🇸🇦"),
            Self::Israel => ("Israel", "IL", "972", "🇮🇱"),
            Self::SouthAfrica => ("South Africa", "ZA", "27", "🇿🇦"),
            Self::Nigeria => ("Nigeria", "NG", "234", "🇳🇬"),
            Self::Kenya => ("Kenya", "KE", "254", "🇰🇪"),
            Self::Egypt => ("Egypt", "EG", "20", "🇪🇬"),
            Self::Morocco => ("Morocco", "MA", "212", "🇲🇦"),
            Self::Brazil => ("Brazil", "BR", "55", "🇧🇷"),
            Self::Argentina => ("Argentina", "AR", "54", "🇦🇷"),
            Self::Chile => ("Chile", "CL", "56", "🇨🇱"),
            Self::Colombia => ("Colombia", "CO", "57", "🇨🇴"),
            Self::Mexico => ("Mexico", "MX", "52", "🇲🇽"),
        };
        CountryData {
            name,
            alpha2,
            dial_code,
            flag,
        }
    }
}
