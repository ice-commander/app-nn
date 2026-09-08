pub fn format(text: &str, indent: usize) -> Result<String, String> {
    let value: serde_json::Value = parse(text)?;
    let pad = vec![b' '; indent];
    let mut out = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(&pad);
    let mut ser = serde_json::Serializer::with_formatter(&mut out, formatter);
    serde::Serialize::serialize(&value, &mut ser).map_err(|e| e.to_string())?;
    String::from_utf8(out).map_err(|e| e.to_string())
}

pub fn minify(text: &str) -> Result<String, String> {
    let value: serde_json::Value = parse(text)?;
    serde_json::to_string(&value).map_err(|e| e.to_string())
}

pub fn validate(text: &str) -> Result<(), String> {
    parse(text).map(|_: serde_json::Value| ())
}

fn parse(text: &str) -> Result<serde_json::Value, String> {
    serde_json::from_str(text).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatting_indents_by_the_width_it_is_given() {
        assert_eq!(format(r#"{"a":1}"#, 2).unwrap(), "{\n  \"a\": 1\n}");
        assert_eq!(format(r#"{"a":1}"#, 4).unwrap(), "{\n    \"a\": 1\n}");
    }

    #[test]
    fn minifying_undoes_formatting() {
        let dense = r#"{"a":[1,2],"b":{"c":null}}"#;
        assert_eq!(minify(&format(dense, 2).unwrap()).unwrap(), dense);
    }

    #[test]
    fn non_latin_content_is_kept_readable_rather_than_escaped() {
        let out = format(r#"{"имя":"дом 🎧"}"#, 2).unwrap();
        assert!(out.contains("дом 🎧"), "got {out}");
    }

    #[test]
    fn a_broken_document_reports_where_it_broke() {
        let err = validate("{\n  \"a\": 1,\n}").unwrap_err();
        assert!(err.contains("line 3"), "unhelpful message: {err}");
    }

    #[test]
    fn a_bare_value_is_valid_json_too() {
        assert!(validate("42").is_ok());
        assert!(validate(r#""text""#).is_ok());
        assert!(validate("").is_err());
    }
}
