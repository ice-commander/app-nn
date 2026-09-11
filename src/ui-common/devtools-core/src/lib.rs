pub mod search;
pub mod tools;

pub use tools::{base64 as base64_tool, hash, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Hash,
    Encoding,
    Format,
}

impl Group {
    pub fn id(self) -> &'static str {
        match self {
            Group::Hash => "hash",
            Group::Encoding => "encoding",
            Group::Format => "format",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    OneLine,
    Document,
}

#[derive(Debug, Clone, Copy)]
pub struct ToolSpec {
    pub id: &'static str,
    pub group: Group,
    pub shape: Shape,
    pub keywords: &'static [&'static str],
    pub takes_file: bool,
}

pub const TOOLS: &[ToolSpec] = &[
    ToolSpec { id: "hash.md5", group: Group::Hash, shape: Shape::OneLine, keywords: &["md5", "checksum", "digest"], takes_file: true },
    ToolSpec { id: "hash.sha1", group: Group::Hash, shape: Shape::OneLine, keywords: &["sha1", "checksum", "digest"], takes_file: true },
    ToolSpec { id: "hash.sha256", group: Group::Hash, shape: Shape::OneLine, keywords: &["sha256", "sha-256", "checksum", "digest"], takes_file: true },
    ToolSpec { id: "hash.sha512", group: Group::Hash, shape: Shape::OneLine, keywords: &["sha512", "sha-512", "checksum", "digest"], takes_file: true },
    ToolSpec { id: "hash.crc32", group: Group::Hash, shape: Shape::OneLine, keywords: &["crc32", "checksum", "digest"], takes_file: true },
    ToolSpec { id: "encoding.base64_encode", group: Group::Encoding, shape: Shape::Document, keywords: &["base64", "b64", "encode"], takes_file: true },
    ToolSpec { id: "encoding.base64_decode", group: Group::Encoding, shape: Shape::Document, keywords: &["base64", "b64", "decode"], takes_file: false },
    ToolSpec { id: "encoding.hex_encode", group: Group::Encoding, shape: Shape::Document, keywords: &["hex", "hexadecimal", "encode"], takes_file: true },
    ToolSpec { id: "encoding.hex_decode", group: Group::Encoding, shape: Shape::Document, keywords: &["hex", "hexadecimal", "decode"], takes_file: false },
    ToolSpec { id: "format.json_pretty", group: Group::Format, shape: Shape::Document, keywords: &["json", "format", "pretty", "indent"], takes_file: true },
    ToolSpec { id: "format.json_minify", group: Group::Format, shape: Shape::Document, keywords: &["json", "minify", "compact"], takes_file: true },
];

pub fn spec(id: &str) -> Option<&'static ToolSpec> {
    TOOLS.iter().find(|t| t.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_tool_has_a_unique_id() {
        let mut ids: Vec<&str> = TOOLS.iter().map(|t| t.id).collect();
        ids.sort_unstable();
        let count = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), count, "two tools share an id");
    }

    #[test]
    fn every_id_carries_its_group_as_a_prefix() {
        for tool in TOOLS {
            assert!(
                tool.id.starts_with(tool.group.id()),
                "{} is not prefixed with {}",
                tool.id,
                tool.group.id()
            );
        }
    }

    #[test]
    fn a_digest_answers_in_one_line_and_a_document_does_not() {
        for tool in TOOLS {
            let expected = if tool.group == Group::Hash { Shape::OneLine } else { Shape::Document };
            assert_eq!(tool.shape, expected, "{}", tool.id);
        }
    }

    #[test]
    fn decoding_takes_pasted_text_rather_than_a_file() {
        for id in ["encoding.base64_decode", "encoding.hex_decode"] {
            assert!(!spec(id).unwrap().takes_file, "{id}");
        }
        assert!(spec("encoding.base64_encode").unwrap().takes_file);
    }

    #[test]
    fn a_tool_is_found_by_its_own_id() {
        assert_eq!(spec("hash.sha256").map(|t| t.group), Some(Group::Hash));
        assert!(spec("nothing.here").is_none());
    }
}
