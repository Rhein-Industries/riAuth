//! Parser entry points compiled only for bounded fuzz campaigns.
use serde_json::Value;

const MAX_INPUT_BYTES: usize = 65_536;
const MAX_STRUCTURED_BYTES: usize = 32_768;
const MAX_JSON_DEPTH: usize = 32;
const MAX_JSON_NODES: usize = 2048;
const MAX_FORM_PAIRS: usize = 128;

pub fn parsers(data: &[u8]) {
    if data.len() > MAX_INPUT_BYTES {
        return;
    }
    crate::radius::fuzz_packet(data);
    let _ = crate::ldap_server::fuzz_ber(data);
    if let Ok(text) = std::str::from_utf8(data) {
        let _ = crate::saml::wire::document(text);
        let _ = jsonwebtoken::decode_header(text);
        if data.len() <= MAX_STRUCTURED_BYTES {
            crate::saml::wire::fuzz_form(text);
            crate::saml::wire::fuzz_verified_redirect(text);
            crate::jose::fuzz_verify_claims(text);
            crate::scim::fuzz_filter(text);
            let pairs: Vec<_> = url::form_urlencoded::parse(data)
                .take(MAX_FORM_PAIRS + 1)
                .map(|(key, value)| (key.into_owned(), value.into_owned()))
                .collect();
            if pairs.len() <= MAX_FORM_PAIRS {
                let _ = crate::oidc::parse_form::<crate::oidc::TokenRequest>(pairs.clone());
                let _ = crate::oidc::parse_form::<crate::oidc::Authorization>(pairs);
                let _ = serde_urlencoded::from_str::<crate::scim::Query>(text);
            }
            if let Some(value) = bounded_json(data) {
                crate::scim::fuzz_resource(value);
                crate::provisioning::fuzz_scim_response(data);
                // Deserialize the original bytes: Value would erase duplicate JWK fields.
                if let Ok(key) = serde_json::from_slice::<crate::jose::PublicJwk>(data) {
                    let _ = key.validate();
                    let _ = crate::dpop::thumbprint(&key);
                }
                if let Ok(keys) = serde_json::from_slice::<crate::jose::PublicJwks>(data) {
                    let _ = keys.validate();
                }
            }
        }
    }
}

// Campaign limits, not additional production acceptance rules. serde_json's
// default recursion limit stays enabled before this iterative work bound.
fn bounded_json(data: &[u8]) -> Option<Value> {
    if data.len() > MAX_STRUCTURED_BYTES {
        return None;
    }
    let value: Value = serde_json::from_slice(data).ok()?;
    let mut pending = vec![(&value, 1usize)];
    let mut nodes = 0;
    while let Some((node, depth)) = pending.pop() {
        nodes += 1;
        if depth > MAX_JSON_DEPTH || nodes > MAX_JSON_NODES {
            return None;
        }
        let children = match node {
            Value::Array(items) => items.len(),
            Value::Object(items) => items.len(),
            _ => 0,
        };
        if children > MAX_JSON_NODES.saturating_sub(nodes + pending.len()) {
            return None;
        }
        match node {
            Value::Array(items) => pending.extend(items.iter().map(|v| (v, depth + 1))),
            Value::Object(items) => pending.extend(items.values().map(|v| (v, depth + 1))),
            _ => {}
        }
    }
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structured_campaign_bounds_depth_nodes_and_bytes() {
        let nested = |levels| format!("{}0{}", "[".repeat(levels), "]".repeat(levels));
        assert!(bounded_json(nested(MAX_JSON_DEPTH - 1).as_bytes()).is_some());
        assert!(bounded_json(nested(MAX_JSON_DEPTH).as_bytes()).is_none());
        assert!(bounded_json(nested(256).as_bytes()).is_none());
        let wide = |items| serde_json::to_vec(&vec![0; items]).unwrap();
        assert!(bounded_json(&wide(MAX_JSON_NODES - 1)).is_some());
        assert!(bounded_json(&wide(MAX_JSON_NODES)).is_none());
        let mut padded = b"0".to_vec();
        padded.resize(MAX_STRUCTURED_BYTES, b' ');
        assert!(bounded_json(&padded).is_some());
        padded.push(b' ');
        assert!(bounded_json(&padded).is_none());
    }
}
