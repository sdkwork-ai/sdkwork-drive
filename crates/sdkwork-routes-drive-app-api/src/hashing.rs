use sdkwork_utils_rust::sha256_hash;

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    format!("sha256:{}", sha256_hash(bytes))
}

pub(crate) fn sha256_raw_hex(bytes: &[u8]) -> String {
    sha256_hash(bytes)
}

pub(crate) fn sha256_raw_hex_separated(parts: &[&[u8]]) -> String {
    let mut buffer = Vec::new();
    for part in parts {
        if !buffer.is_empty() {
            buffer.push(0);
        }
        buffer.extend_from_slice(part);
    }
    sha256_hash(&buffer)
}

pub(crate) fn tenant_shard_prefix(tenant_id: &str) -> String {
    sha256_hash(tenant_id.trim().as_bytes())[..2].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hex_includes_prefix() {
        assert_eq!(
            sha256_hex(b"hello"),
            "sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn sha256_raw_hex_separated_uses_zero_delimiters() {
        let digest = sha256_raw_hex_separated(&[b"tenant", b"node", b"key"]);
        assert_eq!(digest.len(), 64);
        assert_ne!(digest, sha256_raw_hex(b"tenantnodekey"));
    }
}
