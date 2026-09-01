use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

fn registry() -> &'static Mutex<HashSet<String>> {
    static R: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    R.get_or_init(|| Mutex::new(HashSet::new()))
}

pub fn mark_mounted(resource_id: &str) {
    registry().lock().unwrap().insert(resource_id.to_string());
}

pub fn mark_unmounted(resource_id: &str) {
    registry().lock().unwrap().remove(resource_id);
}

pub fn is_mounted(resource_id: &str) -> bool {
    registry().lock().unwrap().contains(resource_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marking_round_trips() {
        assert!(!is_mounted("r1"));
        mark_mounted("r1");
        assert!(is_mounted("r1"));
        assert!(!is_mounted("r2"));
        mark_unmounted("r1");
        assert!(!is_mounted("r1"));
    }

    #[test]
    fn unmarking_an_unknown_id_is_harmless() {
        mark_unmounted("never-mounted");
        assert!(!is_mounted("never-mounted"));
    }
}
