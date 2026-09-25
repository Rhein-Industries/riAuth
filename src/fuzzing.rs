//! Parser entry points compiled only for bounded fuzz campaigns.
pub fn parsers(data: &[u8]) {
    if data.len() > 65_536 {
        return;
    }
    crate::radius::fuzz_packet(data);
    if let Ok(text) = std::str::from_utf8(data) {
        let _ = crate::saml::wire::document(text);
        let _ = jsonwebtoken::decode_header(text);
        if let Ok(key) = serde_json::from_str::<crate::jose::PublicJwk>(text) {
            let _ = key.validate();
            let _ = crate::dpop::thumbprint(&key);
        }
    }
}
