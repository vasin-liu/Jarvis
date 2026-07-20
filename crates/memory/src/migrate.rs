use store::{SourceKind, Store};
use uuid::Uuid;

use crate::error::Result;

const MEMORY_PREFIX: &str = "memory://";

pub fn is_legacy_memory_uri(uri: &str) -> bool {
    let Some(suffix) = uri.strip_prefix(MEMORY_PREFIX) else {
        return false;
    };
    !suffix.is_empty() && suffix.bytes().all(|b| b.is_ascii_digit())
}

pub fn is_uuid_memory_uri(uri: &str) -> bool {
    let Some(suffix) = uri.strip_prefix(MEMORY_PREFIX) else {
        return false;
    };
    Uuid::parse_str(suffix).is_ok()
}

pub fn migrate_legacy_memory_uris(store: &Store) -> Result<usize> {
    let mut migrated = 0usize;
    for source in store.list_sources()? {
        if source.kind != SourceKind::Memory {
            continue;
        }
        if is_uuid_memory_uri(&source.uri) {
            continue;
        }
        if !is_legacy_memory_uri(&source.uri) {
            continue;
        }
        let new_uri = format!("{MEMORY_PREFIX}{}", Uuid::new_v4());
        store.rename_source_id(&source.id, &new_uri)?;
        migrated += 1;
    }
    Ok(migrated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_and_uuid_uri_detection() {
        assert!(is_legacy_memory_uri("memory://1719491234"));
        assert!(!is_legacy_memory_uri("memory://550e8400-e29b-41d4-a716-446655440000"));
        assert!(is_uuid_memory_uri(
            "memory://550e8400-e29b-41d4-a716-446655440000"
        ));
        assert!(!is_uuid_memory_uri("memory://1719491234"));
    }
}
