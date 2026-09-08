use sha1::Digest;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algo {
    Md5,
    Sha1,
    Sha256,
    Sha512,
    Crc32,
}

impl Algo {
    pub fn from_id(id: &str) -> Option<Algo> {
        Some(match id {
            "hash.md5" => Algo::Md5,
            "hash.sha1" => Algo::Sha1,
            "hash.sha256" => Algo::Sha256,
            "hash.sha512" => Algo::Sha512,
            "hash.crc32" => Algo::Crc32,
            _ => return None,
        })
    }
}

pub fn digest(algo: Algo, data: &[u8]) -> String {
    match algo {
        Algo::Md5 => format!("{:x}", md5::compute(data)),
        Algo::Sha1 => hex(&sha1::Sha1::digest(data)),
        Algo::Sha256 => hex(&sha2::Sha256::digest(data)),
        Algo::Sha512 => hex(&sha2::Sha512::digest(data)),
        Algo::Crc32 => format!("{:08x}", crc32fast::hash(data)),
    }
}

pub fn matches(algo: Algo, data: &[u8], expected: &str) -> bool {
    let expected = expected.trim();
    !expected.is_empty() && digest(algo, data).eq_ignore_ascii_case(expected)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const ABC: &[u8] = b"abc";

    #[test]
    fn the_published_digests_of_abc_are_reproduced() {
        assert_eq!(digest(Algo::Md5, ABC), "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(digest(Algo::Sha1, ABC), "a9993e364706816aba3e25717850c26c9cd0d89d");
        assert_eq!(
            digest(Algo::Sha256, ABC),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(digest(Algo::Crc32, ABC), "352441c2");
    }

    #[test]
    fn an_empty_input_still_has_a_digest() {
        assert_eq!(digest(Algo::Md5, b""), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(digest(Algo::Sha256, b"").len(), 64);
    }

    #[test]
    fn bytes_that_are_not_text_are_hashed_all_the_same() {
        let raw = [0u8, 159, 146, 150, 255];
        assert_eq!(digest(Algo::Md5, &raw).len(), 32);
    }

    #[test]
    fn a_pasted_sum_is_compared_regardless_of_case_or_blanks() {
        assert!(matches(Algo::Md5, ABC, "  900150983CD24FB0D6963F7D28E17F72\n"));
        assert!(!matches(Algo::Md5, ABC, "d41d8cd98f00b204e9800998ecf8427e"));
        assert!(!matches(Algo::Md5, ABC, "   "));
    }

    #[test]
    fn every_algorithm_is_reachable_by_the_id_the_list_uses() {
        for id in ["hash.md5", "hash.sha1", "hash.sha256", "hash.sha512", "hash.crc32"] {
            assert!(Algo::from_id(id).is_some(), "{id}");
        }
        assert!(Algo::from_id("hash.sha3").is_none());
    }
}
