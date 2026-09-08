use crate::ToolSpec;

pub fn score(query: &str, spec: &ToolSpec, title: &str) -> Option<u8> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Some(u8::MAX);
    }
    let title = title.to_lowercase();
    let leaf = spec.id.rsplit('.').next().unwrap_or(spec.id);

    if leaf == q || title == q {
        return Some(0);
    }
    if leaf.starts_with(&q) || title.starts_with(&q) {
        return Some(1);
    }
    if spec.keywords.iter().any(|k| k.starts_with(&q)) {
        return Some(2);
    }
    if title.contains(&q) || spec.keywords.iter().any(|k| k.contains(&q)) {
        return Some(3);
    }
    None
}

pub fn matches(query: &str, spec: &ToolSpec, title: &str) -> bool {
    score(query, spec, title).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{spec, TOOLS};

    fn s(id: &str) -> &'static ToolSpec {
        spec(id).unwrap()
    }

    #[test]
    fn an_empty_query_keeps_every_tool() {
        for tool in TOOLS {
            assert!(matches("  ", tool, "whatever"), "{}", tool.id);
        }
    }

    #[test]
    fn the_exact_name_wins_over_a_mere_mention() {
        let exact = score("md5", s("hash.md5"), "MD5").unwrap();
        let mention = score("md5", s("hash.sha1"), "SHA-1");
        assert_eq!(exact, 0);
        assert!(mention.is_none(), "sha1 should not answer to md5");
    }

    #[test]
    fn a_word_the_user_thinks_in_finds_the_tool() {
        assert!(matches("checksum", s("hash.sha256"), "SHA-256"));
        assert!(matches("pretty", s("format.json_pretty"), "JSON"));
        assert!(matches("decode", s("encoding.base64_decode"), "Base64 decode"));
    }

    #[test]
    fn a_prefix_is_enough_and_ranks_above_a_keyword() {
        let prefix = score("sha", s("hash.sha512"), "SHA-512").unwrap();
        let keyword = score("digest", s("hash.sha512"), "SHA-512").unwrap();
        assert!(prefix < keyword, "prefix {prefix} should beat keyword {keyword}");
    }

    #[test]
    fn a_translated_title_is_searched_as_well_as_the_english_id() {
        assert!(matches("хеш", s("hash.md5"), "Хеш MD5"));
    }

    #[test]
    fn something_nobody_offers_matches_nothing() {
        for tool in TOOLS {
            assert!(!matches("zzzz", tool, "title"), "{}", tool.id);
        }
    }
}
