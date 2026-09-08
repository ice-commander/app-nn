use base64::Engine;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alphabet {
    Standard,
    UrlSafe,
}

pub fn encode(alphabet: Alphabet, data: &[u8]) -> String {
    match alphabet {
        Alphabet::Standard => base64::engine::general_purpose::STANDARD.encode(data),
        Alphabet::UrlSafe => base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data),
    }
}

pub fn decode(text: &str) -> Result<Vec<u8>, String> {
    let trimmed: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    use base64::engine::general_purpose as gp;
    let attempts = [
        gp::STANDARD.decode(trimmed.as_bytes()),
        gp::STANDARD_NO_PAD.decode(trimmed.as_bytes()),
        gp::URL_SAFE.decode(trimmed.as_bytes()),
        gp::URL_SAFE_NO_PAD.decode(trimmed.as_bytes()),
    ];
    attempts
        .into_iter()
        .find_map(Result::ok)
        .ok_or_else(|| "not valid base64".to_string())
}

pub fn to_hex(data: &[u8]) -> String {
    data.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn from_hex(text: &str) -> Result<Vec<u8>, String> {
    let cleaned: String = text
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ':' && *c != '-')
        .collect();
    if cleaned.len() % 2 != 0 {
        return Err("hex needs an even number of digits".to_string());
    }
    (0..cleaned.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&cleaned[i..i + 2], 16).map_err(|_| "not valid hex".to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_alphabets_differ_only_where_they_must() {
        let raw = [0xfbu8, 0xff, 0xbe];
        assert_eq!(encode(Alphabet::Standard, &raw), "+/++");
        assert_eq!(encode(Alphabet::UrlSafe, &raw), "-_--");
    }

    #[test]
    fn text_survives_a_round_trip() {
        let text = "дом 家 🎧";
        let encoded = encode(Alphabet::Standard, text.as_bytes());
        assert_eq!(decode(&encoded).unwrap(), text.as_bytes());
    }

    #[test]
    fn padding_and_alphabet_are_worked_out_rather_than_asked_for() {
        for variant in ["aGVsbG8=", "aGVsbG8", "aGVsbG8 =\n"] {
            assert_eq!(decode(variant).unwrap(), b"hello", "{variant:?}");
        }
    }

    #[test]
    fn nonsense_is_refused_rather_than_half_decoded() {
        assert!(decode("не base64!").is_err());
    }

    #[test]
    fn an_empty_input_decodes_to_nothing_instead_of_failing() {
        assert_eq!(decode("   \n").unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn hex_reads_back_what_it_wrote_and_tolerates_separators() {
        assert_eq!(to_hex(&[0, 15, 255]), "000fff");
        assert_eq!(from_hex("00:0f-ff").unwrap(), vec![0, 15, 255]);
        assert_eq!(from_hex("00 0F FF").unwrap(), vec![0, 15, 255]);
    }

    #[test]
    fn half_a_hex_byte_is_an_error_not_a_guess() {
        assert!(from_hex("0f0").is_err());
        assert!(from_hex("zz").is_err());
    }
}
