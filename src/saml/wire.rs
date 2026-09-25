use super::{NameIdFormat, Settings};
use crate::{
    crypto::{self, now},
    error::{Error, Result},
    response::escape,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use roxmltree::{Document, Node};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Read, Write},
};

pub const PROTOCOL: &str = "urn:oasis:names:tc:SAML:2.0:protocol";
pub const ASSERTION: &str = "urn:oasis:names:tc:SAML:2.0:assertion";
pub const METADATA: &str = "urn:oasis:names:tc:SAML:2.0:metadata";
pub const DSIG: &str = "http://www.w3.org/2000/09/xmldsig#";
pub const REDIRECT: &str = "urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect";
pub const POST: &str = "urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST";
pub const RSA256: &str = "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256";
pub const SHA256: &str = "http://www.w3.org/2001/04/xmlenc#sha256";
pub const EXC: &str = "http://www.w3.org/2001/10/xml-exc-c14n#";
pub const ENV: &str = "http://www.w3.org/2000/09/xmldsig#enveloped-signature";
pub const PASSWORD: &str = "urn:oasis:names:tc:SAML:2.0:ac:classes:Password";
pub const PASSWORD_TLS: &str = "urn:oasis:names:tc:SAML:2.0:ac:classes:PasswordProtectedTransport";
const MAX_XML: usize = 48 * 1024;

pub fn bounded_text(s: &str, limit: usize) -> Result<()> {
    if s.is_empty() || s.len() > limit || s.chars().any(char::is_control) {
        return Err(Error::bad("Invalid SAML text value"));
    }
    Ok(())
}
pub fn certificate(pem: &str) -> Result<Vec<u8>> {
    if pem.len() > 16_384 {
        return Err(Error::bad("SAML certificate exceeds 16 KiB"));
    }
    let blocks = pem::parse_many(pem).map_err(|_| Error::bad("Invalid SAML certificate PEM"))?;
    if blocks.len() != 1 || blocks[0].tag() != "CERTIFICATE" {
        return Err(Error::bad("Provide one PEM X.509 certificate"));
    }
    let der = blocks[0].contents();
    let (remaining, cert) = x509_parser::parse_x509_certificate(der)
        .map_err(|_| Error::bad("Invalid SAML X.509 certificate"))?;
    let public = cert
        .public_key()
        .parsed()
        .map_err(|_| Error::bad("Invalid SAML certificate public key"))?;
    let valid = match public {
        x509_parser::public_key::PublicKey::RSA(key) => {
            (2048..=4096).contains(&key.key_size())
                && key.try_exponent().is_ok_and(|e| e >= 65537 && e % 2 == 1)
        }
        _ => false,
    };
    if !remaining.is_empty() || !valid || !cert.validity().is_valid() {
        return Err(Error::bad(
            "SAML certificates must be currently valid RSA 2048..4096 keys",
        ));
    }
    Ok(der.to_vec())
}
pub fn document(xml: &str) -> Result<Document<'_>> {
    if xml.len() > MAX_XML || xml.contains("<!DOCTYPE") || xml.contains("<!ENTITY") {
        return Err(Error::bad("SAML XML size or declaration is not allowed"));
    }
    // roxmltree descends recursively while parsing elements. Bound nesting with
    // risaml's iterative parser before handing untrusted XML to roxmltree.
    risaml::xml::dom::parse_with_limits(
        xml,
        risaml::xml::XmlLimits {
            max_bytes: MAX_XML,
            max_depth: 24,
            max_nodes: 2048,
            ..risaml::xml::XmlLimits::unbounded()
        },
    )
    .map_err(|_| Error::bad("Invalid SAML XML"))?;
    let doc = Document::parse_with_options(
        xml,
        roxmltree::ParsingOptions {
            allow_dtd: false,
            nodes_limit: 2048,
            entity_resolver: None,
        },
    )
    .map_err(|_| Error::bad("Invalid SAML XML"))?;
    for node in doc.descendants() {
        if node.is_comment()
            || node.is_pi()
            || node.ancestors().count() > 24
            || node.attributes().len() > 32
            || node.attributes().any(|a| a.value().len() > 8192)
        {
            return Err(Error::bad(
                "SAML XML exceeds this profile's structural limits",
            ));
        }
    }
    Ok(doc)
}

#[cfg(test)]
mod document_tests {
    use super::document;

    #[test]
    fn bounds_depth_before_dom_parsing() {
        let nested = |depth: usize| format!("{}{}", "<a>".repeat(depth), "</a>".repeat(depth));
        assert!(document(&nested(23)).is_ok());
        assert!(document(&nested(24)).is_err());
        assert!(document(&nested(25)).is_err());
    }

    #[test]
    fn rejects_retained_deep_unclosed_xml() {
        let xml = include_str!("fixtures/xml-deep-unclosed");
        assert_eq!(xml.len(), 3264);
        assert!(document(xml).is_err());
    }

    #[test]
    fn cdata_markup_does_not_count_as_elements() {
        assert!(document("<a><![CDATA[<b>]]></a>").is_ok());
    }
}
pub(crate) fn attributes(node: Node<'_, '_>, allowed: &[&str]) -> Result<()> {
    if node
        .attributes()
        .any(|a| a.namespace().is_some() || !allowed.contains(&a.name()))
    {
        return Err(Error::bad("Unsupported SAML attribute"));
    }
    Ok(())
}
pub(crate) fn text(node: Node<'_, '_>) -> Result<String> {
    text_with_limit(node, 2048)
}
fn text_with_limit(node: Node<'_, '_>, limit: usize) -> Result<String> {
    if node.children().any(|n| !n.is_text()) {
        return Err(Error::bad("SAML text cannot contain nested elements"));
    }
    let value = node.children().filter_map(|n| n.text()).collect::<String>();
    bounded_text(&value, limit)?;
    Ok(value)
}
pub(crate) fn child<'a, 'i>(
    node: Node<'a, 'i>,
    ns: &str,
    name: &str,
) -> Result<Option<Node<'a, 'i>>> {
    let values = node
        .children()
        .filter(|n| n.has_tag_name((ns, name)))
        .collect::<Vec<_>>();
    if values.len() > 1 {
        return Err(Error::bad("Duplicate SAML element"));
    }
    Ok(values.into_iter().next())
}
fn bool_attr(node: Node<'_, '_>, name: &str) -> Result<bool> {
    match node.attribute(name) {
        None | Some("false" | "0") => Ok(false),
        Some("true" | "1") => Ok(true),
        _ => Err(Error::bad("Invalid SAML boolean")),
    }
}
pub fn timestamp(at: u64) -> Result<String> {
    time::OffsetDateTime::from_unix_timestamp(i64::try_from(at).map_err(Error::internal)?)
        .map_err(Error::internal)?
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(Error::internal)
}
pub(crate) fn parse_time(value: &str) -> Result<u64> {
    let at = time::OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339)
        .map_err(|_| Error::bad("Invalid SAML timestamp"))?
        .unix_timestamp();
    u64::try_from(at).map_err(|_| Error::bad("Invalid SAML timestamp"))
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Authn {
    pub id: Option<String>,
    pub acs: String,
    pub relay_state: Option<String>,
    pub force: bool,
    pub passive: bool,
    pub contexts: Vec<String>,
    pub minimum: bool,
    pub name_id_format: NameIdFormat,
    pub requested_subject: Option<String>,
    pub allow_create: bool,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Logout {
    pub id: String,
    pub name_id: String,
    pub format: Option<String>,
    pub indices: Vec<String>,
    pub relay_state: Option<String>,
}
pub enum Message {
    Authn(Authn),
    Logout(Logout),
}

// The request/response selector is only a routing hint. The complete field set and
// signature are validated before any stored logout state is read or changed.
pub(crate) fn is_response(raw: &str) -> bool {
    raw.split('&').any(|pair| pair.starts_with("SAMLResponse="))
}

fn logout_envelope(root: Node<'_, '_>, kind: &str, endpoint: &str, peer: &str) -> Result<()> {
    if !root.has_tag_name((PROTOCOL, kind))
        || root.attribute("Version") != Some("2.0")
        || root.attribute("Destination") != Some(endpoint)
    {
        return Err(Error::bad("SAML logout envelope mismatch"));
    }
    let instant = parse_time(
        root.attribute("IssueInstant")
            .ok_or_else(|| Error::bad("Missing SAML IssueInstant"))?,
    )?;
    if instant > now() + 30 || instant.saturating_add(300) < now() {
        return Err(Error::bad("SAML logout is expired or future dated"));
    }
    let issuer =
        child(root, ASSERTION, "Issuer")?.ok_or_else(|| Error::bad("Missing SAML issuer"))?;
    attributes(issuer, &["Format"])?;
    if issuer
        .attribute("Format")
        .is_some_and(|v| v != "urn:oasis:names:tc:SAML:2.0:nameid-format:entity")
        || text(issuer)? != peer
    {
        return Err(Error::forbidden());
    }
    Ok(())
}

pub(crate) fn receive_logout(
    certificates: &[String],
    endpoint: &str,
    peer: &str,
    name_qualifier: &str,
    sp_qualifier: &str,
    raw: &str,
    post: bool,
) -> Result<Logout> {
    let (xml, relay_state) = verified_xml(certificates, raw, post, "SAMLRequest")?;
    let doc = document(&xml)?;
    let root = doc.root_element();
    logout_envelope(root, "LogoutRequest", endpoint, peer)?;
    attributes(
        root,
        &[
            "ID",
            "Version",
            "IssueInstant",
            "Destination",
            "NotOnOrAfter",
            "Reason",
        ],
    )?;
    let instant = parse_time(root.attribute("IssueInstant").unwrap())?;
    if let Some(expiry) = root.attribute("NotOnOrAfter")
        && (parse_time(expiry)? <= now() || parse_time(expiry)? > instant + 300)
    {
        return Err(Error::bad("Invalid SAML logout expiry"));
    }
    for node in root.children() {
        if node.is_text() && node.text().is_none_or(|s| s.trim().is_empty()) {
            continue;
        }
        if ![
            (ASSERTION, "Issuer"),
            (DSIG, "Signature"),
            (ASSERTION, "NameID"),
            (PROTOCOL, "SessionIndex"),
        ]
        .iter()
        .any(|tag| node.has_tag_name(*tag))
        {
            return Err(Error::bad("Unsupported SAML logout element"));
        }
    }
    let subject = child(root, ASSERTION, "NameID")?
        .ok_or_else(|| Error::bad("SAML logout requires NameID"))?;
    attributes(subject, &["Format", "NameQualifier", "SPNameQualifier"])?;
    if subject
        .attribute("NameQualifier")
        .is_some_and(|v| v != name_qualifier)
        || subject
            .attribute("SPNameQualifier")
            .is_some_and(|v| v != sp_qualifier)
    {
        return Err(Error::forbidden());
    }
    let indices = root
        .children()
        .filter(|n| n.has_tag_name((PROTOCOL, "SessionIndex")))
        .map(|n| {
            attributes(n, &[])?;
            text(n)
        })
        .collect::<Result<Vec<_>>>()?;
    if indices.is_empty()
        || indices.len() > 8
        || indices.iter().collect::<BTreeSet<_>>().len() != indices.len()
    {
        return Err(Error::bad(
            "SAML logout requires one to eight distinct session indices",
        ));
    }
    Ok(Logout {
        id: root.attribute("ID").unwrap().into(),
        name_id: text(subject)?,
        format: subject.attribute("Format").map(String::from),
        indices,
        relay_state,
    })
}

pub(crate) struct LogoutResponse {
    pub request: String,
    pub relay_state: String,
    pub success: bool,
}
pub(crate) fn receive_logout_response(
    certificates: &[String],
    endpoint: &str,
    peer: &str,
    raw: &str,
    post: bool,
) -> Result<LogoutResponse> {
    let (xml, relay) = verified_xml(certificates, raw, post, "SAMLResponse")?;
    let doc = document(&xml)?;
    let root = doc.root_element();
    logout_envelope(root, "LogoutResponse", endpoint, peer)?;
    attributes(
        root,
        &[
            "ID",
            "Version",
            "IssueInstant",
            "Destination",
            "InResponseTo",
        ],
    )?;
    for node in root.children() {
        if node.is_text() && node.text().is_none_or(|s| s.trim().is_empty()) {
            continue;
        }
        if ![
            (ASSERTION, "Issuer"),
            (DSIG, "Signature"),
            (PROTOCOL, "Status"),
        ]
        .iter()
        .any(|tag| node.has_tag_name(*tag))
        {
            return Err(Error::bad("Unsupported SAML logout response element"));
        }
    }
    let status =
        child(root, PROTOCOL, "Status")?.ok_or_else(|| Error::bad("Missing SAML logout status"))?;
    attributes(status, &[])?;
    let code = child(status, PROTOCOL, "StatusCode")?
        .ok_or_else(|| Error::bad("Missing SAML logout status code"))?;
    let message = child(status, PROTOCOL, "StatusMessage")?;
    if let Some(message) = message {
        attributes(message, &[])?;
        text(message)?;
    }
    for node in status.children() {
        if node.is_text() && node.text().is_none_or(|s| s.trim().is_empty()) {
            continue;
        }
        if node != code && Some(node) != message {
            return Err(Error::bad("Unsupported SAML logout status"));
        }
    }
    for node in code.descendants() {
        if node.is_text() && node.text().is_none_or(|s| s.trim().is_empty()) {
            continue;
        }
        if !node.has_tag_name((PROTOCOL, "StatusCode")) || node.ancestors().count() > 8 {
            return Err(Error::bad("Invalid SAML logout status"));
        }
        attributes(node, &["Value"])?;
        let value = node
            .attribute("Value")
            .ok_or_else(|| Error::bad("Missing SAML status value"))?;
        bounded_text(value, 256)?;
        url::Url::parse(value).map_err(|_| Error::bad("Invalid SAML status URI"))?;
        child(node, PROTOCOL, "StatusCode")?;
    }
    let success = code.attribute("Value") == Some("urn:oasis:names:tc:SAML:2.0:status:Success")
        && !code.children().any(|n| n.is_element());
    let request = root
        .attribute("InResponseTo")
        .ok_or_else(|| Error::bad("Unsolicited logout response"))?
        .to_owned();
    let relay_state = relay
        .filter(|v| v.len() == 43)
        .ok_or_else(|| Error::bad("Missing SAML logout correlation"))?;
    Ok(LogoutResponse {
        request,
        relay_state,
        success,
    })
}

fn fields(raw: &str, post: bool, field: &str) -> Result<BTreeMap<String, (String, String)>> {
    if raw.len() > 64 * 1024 {
        return Err(Error::bad("SAML request exceeds 64 KiB"));
    }
    let allowed = if post {
        &[field, "RelayState"][..]
    } else {
        &[field, "RelayState", "SigAlg", "Signature"][..]
    };
    let mut fields = BTreeMap::new();
    for pair in raw.split('&') {
        let (name, value) = pair
            .split_once('=')
            .ok_or_else(|| Error::bad("Invalid SAML form"))?;
        if !allowed.contains(&name) || fields.contains_key(name) {
            return Err(Error::bad("Unexpected or duplicate SAML form field"));
        }
        let bytes = value.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'%' {
                if i + 2 >= bytes.len()
                    || !bytes[i + 1].is_ascii_hexdigit()
                    || !bytes[i + 2].is_ascii_hexdigit()
                {
                    return Err(Error::bad("Invalid SAML percent encoding"));
                }
                i += 2;
            }
            i += 1;
        }
        let decoded = url::form_urlencoded::parse(pair.as_bytes())
            .next()
            .ok_or_else(|| Error::bad("Invalid SAML encoding"))?
            .1
            .into_owned();
        if decoded.contains('\u{fffd}') {
            return Err(Error::bad("SAML form fields must be UTF-8"));
        }
        fields.insert(name.into(), (value.into(), decoded));
    }
    if let Some((_, state)) = fields.get("RelayState")
        && !state.is_empty()
    {
        bounded_text(state, 80)?;
    }
    Ok(fields)
}
pub(crate) fn verified_xml(
    trusted_certificates: &[String],
    raw: &str,
    post: bool,
    field: &str,
) -> Result<(String, Option<String>)> {
    if trusted_certificates.is_empty() || trusted_certificates.len() > 4 {
        return Err(Error::forbidden());
    }
    for cert in trusted_certificates {
        certificate(cert)?;
    }
    let fields = fields(raw, post, field)?;
    let request = fields
        .get(field)
        .ok_or_else(|| Error::bad("Missing SAML message"))?;
    let bytes = STANDARD
        .decode(&request.1)
        .map_err(|_| Error::bad("Invalid SAMLRequest base64"))?;
    let xml = if post {
        String::from_utf8(bytes).map_err(|_| Error::bad("SAML XML must be UTF-8"))?
    } else {
        let algorithm = fields
            .get("SigAlg")
            .filter(|(_, v)| v == RSA256)
            .ok_or_else(|| Error::bad("SAML Redirect requires RSA-SHA256"))?;
        let signature = &fields
            .get("Signature")
            .ok_or_else(|| Error::bad("Signed SAML requests are required"))?
            .1;
        let mut signed = format!("{field}={}", request.0);
        if let Some((raw, _)) = fields.get("RelayState") {
            signed.push_str(&format!("&RelayState={raw}"));
        }
        signed.push_str(&format!("&SigAlg={}", algorithm.0));
        let trusted = trusted_certificates.iter().any(|cert| {
            risaml::crypto::verify_message_signature(&signed, signature, cert, RSA256)
                .unwrap_or(false)
        });
        if !trusted {
            return Err(Error::forbidden());
        }
        let mut decoder = flate2::read::DeflateDecoder::new(bytes.as_slice());
        let mut xml = String::new();
        decoder
            .by_ref()
            .take((MAX_XML + 1) as u64)
            .read_to_string(&mut xml)
            .map_err(|_| Error::bad("Invalid compressed SAML XML"))?;
        if decoder.total_in() != bytes.len() as u64 {
            return Err(Error::bad("Trailing or excessive SAML DEFLATE data"));
        }
        xml
    };
    let doc = document(&xml)?;
    let root = doc.root_element();
    let id = root
        .attribute("ID")
        .ok_or_else(|| Error::bad("SAML request requires an ID"))?;
    if id.is_empty()
        || id.len() > 128
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-.".contains(&b))
        || !id.as_bytes()[0].is_ascii_alphabetic() && id.as_bytes()[0] != b'_'
    {
        return Err(Error::bad("Invalid SAML request ID"));
    }
    if doc
        .descendants()
        .filter(|n| n.is_element() && *n != root)
        .any(|n| {
            n.attributes().any(|a| {
                a.name() == "ID"
                    || a.name() == "Id"
                    || a.name() == "id"
                    || a.name() == "AssertionID"
            })
        })
    {
        return Err(Error::bad(
            "Nested IDs are not allowed in this SAML request profile",
        ));
    }
    let signatures = doc
        .descendants()
        .filter(|n| n.has_tag_name((DSIG, "Signature")))
        .collect::<Vec<_>>();
    if post {
        if signatures.len() != 1 || signatures[0].parent() != Some(root) {
            return Err(Error::bad("SAML POST requires one root signature"));
        }
        let sig = signatures[0];
        let references = sig
            .descendants()
            .filter(|n| n.has_tag_name((DSIG, "Reference")))
            .collect::<Vec<_>>();
        if references.len() != 1
            || references[0].attribute("URI") != Some(format!("#{id}").as_str())
        {
            return Err(Error::bad("SAML signature must cover the complete request"));
        }
        let transforms = references[0]
            .descendants()
            .filter(|n| n.has_tag_name((DSIG, "Transform")))
            .map(|n| n.attribute("Algorithm"))
            .collect::<Vec<_>>();
        if transforms != vec![Some(ENV), Some(EXC)] {
            return Err(Error::bad("Unsupported SAML signature transforms"));
        }
        for node in sig.descendants().filter(Node::is_element) {
            if node.tag_name().namespace() != Some(DSIG)
                || ![
                    "Signature",
                    "SignedInfo",
                    "CanonicalizationMethod",
                    "SignatureMethod",
                    "Reference",
                    "Transforms",
                    "Transform",
                    "DigestMethod",
                    "DigestValue",
                    "SignatureValue",
                    "KeyInfo",
                    "KeyName",
                    "X509Data",
                    "X509Certificate",
                ]
                .contains(&node.tag_name().name())
            {
                return Err(Error::bad("Unsupported SAML signature structure"));
            }
            if let Some(expected) = match node.tag_name().name() {
                "CanonicalizationMethod" => Some(EXC),
                "SignatureMethod" => Some(RSA256),
                "DigestMethod" => Some(SHA256),
                _ => None,
            } && node.attribute("Algorithm") != Some(expected)
            {
                return Err(Error::bad("Unsupported SAML signature algorithm"));
            }
        }
        let limits = risaml::xml::XmlLimits {
            max_bytes: MAX_XML,
            max_depth: 24,
            max_nodes: 2048,
            max_attributes_per_element: 32,
            max_attribute_value_bytes: 8192,
            max_text_bytes: 16384,
        };
        let (valid, covered) =
            risaml::crypto::verify_signature_with_limits(&xml, trusted_certificates, limits)
                .map_err(|_| Error::forbidden())?;
        if !valid || covered.as_deref() != Some(&xml[root.range()]) {
            return Err(Error::forbidden());
        }
    } else if !signatures.is_empty() {
        return Err(Error::bad(
            "Redirect requests cannot contain XML signatures",
        ));
    }
    Ok((xml, fields.get("RelayState").map(|(_, s)| s.clone())))
}
pub fn receive(
    settings: &Settings,
    endpoint: &str,
    issuer_id: &str,
    raw: &str,
    post: bool,
) -> Result<Message> {
    let (xml, relay_state) = verified_xml(&settings.sp_certificates_pem, raw, post, "SAMLRequest")?;
    let doc = document(&xml)?;
    let root = doc.root_element();
    if root.tag_name().namespace() != Some(PROTOCOL)
        || !matches!(root.tag_name().name(), "AuthnRequest" | "LogoutRequest")
    {
        return Err(Error::bad("Expected SAML AuthnRequest or LogoutRequest"));
    }
    if root.attribute("Version") != Some("2.0") || root.attribute("Destination") != Some(endpoint) {
        return Err(Error::bad("SAML version or destination mismatch"));
    }
    let instant = parse_time(
        root.attribute("IssueInstant")
            .ok_or_else(|| Error::bad("Missing SAML IssueInstant"))?,
    )?;
    if instant > now() + 30 || instant + 300 < now() {
        return Err(Error::bad("SAML request is expired or future dated"));
    }
    let issuer =
        child(root, ASSERTION, "Issuer")?.ok_or_else(|| Error::bad("Missing SAML issuer"))?;
    attributes(issuer, &["Format"])?;
    if issuer
        .attribute("Format")
        .is_some_and(|v| v != "urn:oasis:names:tc:SAML:2.0:nameid-format:entity")
        || text(issuer)? != settings.sp_entity_id
    {
        return Err(Error::forbidden());
    }
    let id = root.attribute("ID").unwrap().to_owned();
    let mut seen = BTreeSet::new();
    for node in root.children() {
        if node.is_text() {
            if node.text().is_some_and(|t| !t.trim().is_empty()) {
                return Err(Error::bad("Unexpected SAML text"));
            }
            continue;
        }
        let name = (
            node.tag_name().namespace().unwrap_or(""),
            node.tag_name().name(),
        );
        let allowed = if root.tag_name().name() == "AuthnRequest" {
            [
                (ASSERTION, "Issuer"),
                (DSIG, "Signature"),
                (PROTOCOL, "NameIDPolicy"),
                (PROTOCOL, "RequestedAuthnContext"),
                (ASSERTION, "Subject"),
            ]
            .as_slice()
            .to_vec()
        } else {
            [
                (ASSERTION, "Issuer"),
                (DSIG, "Signature"),
                (ASSERTION, "NameID"),
                (PROTOCOL, "SessionIndex"),
            ]
            .as_slice()
            .to_vec()
        };
        if !allowed.contains(&name) || name.1 != "SessionIndex" && !seen.insert(name) {
            return Err(Error::bad("Unsupported or duplicate SAML request element"));
        }
    }
    if root.tag_name().name() == "LogoutRequest" {
        attributes(
            root,
            &[
                "ID",
                "Version",
                "IssueInstant",
                "Destination",
                "NotOnOrAfter",
                "Reason",
            ],
        )?;
        if let Some(expiry) = root.attribute("NotOnOrAfter")
            && (parse_time(expiry)? <= now() || parse_time(expiry)? > instant + 300)
        {
            return Err(Error::bad("Invalid SAML logout expiry"));
        }
        let subject = child(root, ASSERTION, "NameID")?
            .ok_or_else(|| Error::bad("SAML logout requires NameID"))?;
        attributes(subject, &["Format", "NameQualifier", "SPNameQualifier"])?;
        if subject
            .attribute("NameQualifier")
            .is_some_and(|v| v != issuer_id)
            || subject
                .attribute("SPNameQualifier")
                .is_some_and(|v| v != settings.sp_entity_id)
        {
            return Err(Error::forbidden());
        }
        let indices = root
            .children()
            .filter(|n| n.has_tag_name((PROTOCOL, "SessionIndex")))
            .map(|n| {
                attributes(n, &[])?;
                text(n)
            })
            .collect::<Result<Vec<_>>>()?;
        if indices.is_empty() || indices.len() > 8 {
            return Err(Error::bad(
                "SAML logout requires one to eight session indices",
            ));
        }
        return Ok(Message::Logout(Logout {
            id,
            name_id: text(subject)?,
            format: subject.attribute("Format").map(String::from),
            indices,
            relay_state,
        }));
    }
    attributes(
        root,
        &[
            "ID",
            "Version",
            "IssueInstant",
            "Destination",
            "ForceAuthn",
            "IsPassive",
            "ProtocolBinding",
            "AssertionConsumerServiceURL",
            "AssertionConsumerServiceIndex",
            "ProviderName",
            "AttributeConsumingServiceIndex",
        ],
    )?;
    if root.attribute("AttributeConsumingServiceIndex").is_some()
        || root.attribute("ProtocolBinding").is_some_and(|b| b != POST)
    {
        return Err(Error::bad(
            "SAML profile requires HTTP-POST responses and configured attribute mappings",
        ));
    }
    let acs = match (
        root.attribute("AssertionConsumerServiceURL"),
        root.attribute("AssertionConsumerServiceIndex"),
    ) {
        (Some(url), None) => settings
            .acs_urls
            .iter()
            .find(|s| *s == url)
            .cloned()
            .ok_or_else(|| Error::bad("Unregistered SAML ACS"))?,
        (None, Some(index)) => {
            let index = index
                .parse::<u16>()
                .map_err(|_| Error::bad("Invalid SAML ACS index"))?;
            if settings.acs_indices.is_empty() {
                settings.acs_urls.get(index as usize)
            } else {
                settings.acs_indices.get(&index)
            }
            .cloned()
            .ok_or_else(|| Error::bad("Unknown SAML ACS index"))?
        }
        (None, None) => settings.acs_urls[0].clone(),
        _ => {
            return Err(Error::bad(
                "SAML request cannot specify both ACS URL and index",
            ));
        }
    };
    let mut allow_create = true;
    if let Some(policy) = child(root, PROTOCOL, "NameIDPolicy")? {
        attributes(policy, &["Format", "SPNameQualifier", "AllowCreate"])?;
        if policy.children().any(|n| n.is_element())
            || policy
                .attribute("SPNameQualifier")
                .is_some_and(|v| v != settings.sp_entity_id)
            || policy.attribute("Format").is_some_and(|v| {
                v != settings.name_id_format.uri()
                    && v != "urn:oasis:names:tc:SAML:1.1:nameid-format:unspecified"
            })
        {
            return Err(Error::bad("Unsupported SAML NameID policy"));
        }
        allow_create = bool_attr(policy, "AllowCreate")?;
    }
    let (contexts, minimum) = if let Some(context) = child(root, PROTOCOL, "RequestedAuthnContext")?
    {
        attributes(context, &["Comparison"])?;
        let minimum = match context.attribute("Comparison") {
            None | Some("exact") => false,
            Some("minimum") => true,
            _ => {
                return Err(Error::bad(
                    "SAML supports exact/minimum requested authentication context",
                ));
            }
        };
        let mut contexts = Vec::new();
        for node in context.children().filter(Node::is_element) {
            if !node.has_tag_name((ASSERTION, "AuthnContextClassRef")) {
                return Err(Error::bad("Unsupported SAML authentication context"));
            }
            attributes(node, &[])?;
            let value = text(node)?;
            if ![
                PASSWORD,
                PASSWORD_TLS,
                crate::assurance::MFA,
                crate::assurance::FEDERATED,
            ]
            .contains(&value.as_str())
            {
                return Err(Error::bad("Unsupported SAML authentication context class"));
            }
            contexts.push(value);
        }
        if contexts.is_empty() || contexts.len() > 8 {
            return Err(Error::bad("Invalid SAML authentication context list"));
        }
        (contexts, minimum)
    } else {
        (vec![], false)
    };
    let requested_subject = if let Some(subject) = child(root, ASSERTION, "Subject")? {
        attributes(subject, &[])?;
        let name = child(subject, ASSERTION, "NameID")?
            .ok_or_else(|| Error::bad("SAML subject requires NameID"))?;
        if subject.children().filter(Node::is_element).count() != 1 {
            return Err(Error::bad("Unsupported SAML subject confirmation"));
        }
        attributes(name, &["Format"])?;
        if name
            .attribute("Format")
            .is_some_and(|v| v != settings.name_id_format.uri())
        {
            return Err(Error::bad("Requested SAML subject format mismatch"));
        }
        Some(text(name)?)
    } else {
        None
    };
    let force = bool_attr(root, "ForceAuthn")?;
    let passive = bool_attr(root, "IsPassive")?;
    Ok(Message::Authn(Authn {
        id: Some(id),
        acs,
        relay_state,
        force,
        passive,
        contexts,
        minimum,
        name_id_format: settings.name_id_format.clone(),
        requested_subject,
        allow_create,
    }))
}

pub fn response_xml(
    issuer: &str,
    target: &str,
    in_response_to: Option<&str>,
    status: &str,
    assertion: Option<&str>,
) -> Result<String> {
    let id = format!("_{}", crypto::id());
    let instant = timestamp(now())?;
    let correlation = in_response_to
        .map(|v| format!(" InResponseTo=\"{}\"", escape(v)))
        .unwrap_or_default();
    let code = if status == "Success" {
        "<samlp:StatusCode Value=\"urn:oasis:names:tc:SAML:2.0:status:Success\"/>".to_string()
    } else {
        format!(
            "<samlp:StatusCode Value=\"urn:oasis:names:tc:SAML:2.0:status:Responder\"><samlp:StatusCode Value=\"urn:oasis:names:tc:SAML:2.0:status:{}\"/></samlp:StatusCode>",
            escape(status)
        )
    };
    Ok(format!(
        "<samlp:Response xmlns:samlp=\"{PROTOCOL}\" xmlns:saml=\"{ASSERTION}\" ID=\"{id}\" Version=\"2.0\" IssueInstant=\"{instant}\" Destination=\"{}\"{correlation}><saml:Issuer>{}</saml:Issuer><samlp:Status>{code}</samlp:Status>{}</samlp:Response>",
        escape(target),
        escape(issuer),
        assertion.unwrap_or("")
    ))
}
pub(crate) fn redirect_message(
    target: &str,
    xml: &str,
    relay: Option<&str>,
    key_pem: &str,
    field: &str,
) -> Result<String> {
    let mut compressor =
        flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::fast());
    compressor
        .write_all(xml.as_bytes())
        .map_err(Error::internal)?;
    let encoded = STANDARD.encode(compressor.finish().map_err(Error::internal)?);
    let mut pairs = vec![(field, encoded)];
    if let Some(relay) = relay {
        pairs.push(("RelayState", relay.into()));
    }
    pairs.push(("SigAlg", RSA256.into()));
    let mut query = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(pairs)
        .finish();
    let key = risaml::crypto::keys::load_private_key(key_pem, None).map_err(Error::internal)?;
    let signature = risaml::crypto::construct_message_signature(&query, &key, RSA256)
        .map_err(Error::internal)?;
    query.push('&');
    query.push_str(
        &url::form_urlencoded::Serializer::new(String::new())
            .append_pair("Signature", &signature)
            .finish(),
    );
    Ok(format!("{target}?{query}"))
}

/// Offline, explicitly selected SP metadata. No network metadata/KeyInfo resolution.
pub fn import_metadata(
    xml: &str,
    expected: &str,
    idp_certificate_pem: String,
) -> Result<serde_json::Value> {
    let doc = document(xml)?;
    let root = doc.root_element();
    if !root.has_tag_name((METADATA, "EntityDescriptor"))
        || root.attribute("entityID") != Some(expected)
    {
        return Err(Error::bad(
            "Select one EntityDescriptor with the exact expected SP entity ID",
        ));
    }
    let descriptor = child(root, METADATA, "SPSSODescriptor")?
        .ok_or_else(|| Error::bad("Metadata has no unique SPSSODescriptor"))?;
    if !descriptor
        .attribute("protocolSupportEnumeration")
        .is_some_and(|v| v.split_whitespace().any(|p| p == PROTOCOL))
    {
        return Err(Error::bad("Metadata does not advertise SAML 2.0"));
    }
    let mut indexed = BTreeMap::new();
    let mut default = None;
    let mut certificates = Vec::new();
    let mut encryption = None;
    let mut slo_redirect = None;
    let mut slo_post = None;
    let mut formats = Vec::new();
    let mut unresolved = Vec::new();
    for node in descriptor.children().filter(Node::is_element) {
        match node.tag_name(){
        tag if tag.namespace()==Some(METADATA)&&tag.name()=="AssertionConsumerService"=>{
            if node.attribute("Binding")!=Some(POST){unresolved.push("A non-POST ACS was omitted from this HTTP-POST response profile");continue;}
            let url=node.attribute("Location").ok_or_else(||Error::bad("ACS Location is required"))?;super::endpoint_url(url)?;
            let index=node.attribute("index").and_then(|s|s.parse::<u16>().ok()).ok_or_else(||Error::bad("ACS index must be an unsigned 16-bit integer"))?;
            if indexed.insert(index,url.to_owned()).is_some(){return Err(Error::bad("Duplicate ACS index"));}
            if bool_attr(node,"isDefault")? && default.replace(url.to_owned()).is_some(){return Err(Error::bad("Multiple default ACS entries"));}
        },
        tag if tag.namespace()==Some(METADATA)&&tag.name()=="KeyDescriptor"=>{
            let key_info=child(node,DSIG,"KeyInfo")?.ok_or_else(||Error::bad("SAML metadata requires embedded X.509 keys"))?;
            let data=child(key_info,DSIG,"X509Data")?.ok_or_else(||Error::bad("SAML metadata requires X509Data"))?;
            let nodes=data.children().filter(|n|n.has_tag_name((DSIG,"X509Certificate"))).collect::<Vec<_>>();if nodes.is_empty()||nodes.len()>4{return Err(Error::bad("Invalid metadata certificate list"));}
            for node in nodes {
                if node.children().any(|n| !n.is_text()) {return Err(Error::bad("Metadata certificates must contain only base64 text"));}
                let raw=node.children().filter_map(|n|n.text()).collect::<String>();
                if raw.len()>16384 {return Err(Error::bad("Metadata certificate exceeds 16 KiB"));}
                let der=STANDARD.decode(raw.split_whitespace().collect::<String>()).map_err(|_|Error::bad("Invalid metadata certificate base64"))?;let pem=pem::encode(&pem::Pem::new("CERTIFICATE",der));certificate(&pem)?;
                match node.parent().and_then(|n|n.parent()).and_then(|n|n.parent()).and_then(|n|n.attribute("use")) {
                    Some("signing")=>{if !certificates.contains(&pem){certificates.push(pem);}},
                    Some("encryption")=>{if encryption.as_ref().is_some_and(|c|*c!=pem){return Err(Error::bad("Choose one SP encryption certificate explicitly"));}encryption=Some(pem);},
                    None=>{if !certificates.contains(&pem){certificates.push(pem.clone());}
                    if encryption.is_none(){encryption=Some(pem);}},
                    _=>return Err(Error::bad("Unsupported KeyDescriptor use")),
                }
            }
        },
        tag if tag.namespace()==Some(METADATA)&&tag.name()=="SingleLogoutService"=>{
            if node.attribute("ResponseLocation").is_some_and(|v|Some(v)!=node.attribute("Location")){return Err(Error::bad("Separate SLO ResponseLocation requires explicit configuration"));}
            let url=node.attribute("Location").ok_or_else(||Error::bad("Missing SLO Location"))?;super::endpoint_url(url)?;
            let entry=match node.attribute("Binding"){Some(REDIRECT)=>&mut slo_redirect,Some(POST)=>&mut slo_post,_=>{unresolved.push("An unsupported SLO binding was omitted");continue;}};
            if entry.replace(url.to_owned()).is_some(){return Err(Error::bad("Choose one SLO endpoint per binding"));}
        },
        tag if tag.namespace()==Some(METADATA)&&tag.name()=="NameIDFormat"=>formats.push(text(node)?),
        tag if tag.namespace()==Some(METADATA)&&tag.name()=="AttributeConsumingService"=>unresolved.push("Translate AttributeConsumingService requested attributes into reviewed scoped claim mappings"),
        tag if tag.namespace()==Some(METADATA)&&tag.name()=="ArtifactResolutionService"=>unresolved.push("Artifact binding is outside this profile"),
        tag if tag.namespace()==Some(METADATA)&&tag.name()=="Extensions"=>unresolved.push("Review SP metadata extensions"),
        _=>{},
    }
    }
    if indexed.is_empty() || indexed.len() > 10 || certificates.is_empty() || certificates.len() > 4
    {
        return Err(Error::bad(
            "Metadata needs 1..10 POST ACS endpoints and 1..4 pinned signing certificates",
        ));
    }
    certificate(&idp_certificate_pem)?;
    let mut urls = Vec::new();
    if let Some(default) = default {
        urls.push(default);
    }
    for url in indexed.values() {
        if !urls.contains(url) {
            urls.push(url.clone());
        }
    }
    let format = [
        NameIdFormat::Persistent,
        NameIdFormat::Transient,
        NameIdFormat::Email,
        NameIdFormat::Unspecified,
    ]
    .into_iter()
    .find(|format| formats.is_empty() || formats.iter().any(|f| f == format.uri()))
    .ok_or_else(|| Error::bad("SP does not advertise a supported NameID format"))?;
    // Import does not silently enable encryption merely because metadata contains a key.
    let settings = Settings {
        sp_entity_id: expected.into(),
        acs_urls: urls,
        acs_indices: indexed,
        idp_entity_id: None,
        idp_certificate_pem,
        sp_certificates_pem: certificates,
        encryption_certificate_pem: None,
        name_id_format: format,
        attributes: vec![],
        slo_redirect_url: slo_redirect,
        slo_post_url: slo_post,
        idp_initiated: false,
        default_relay_state: None,
        assertion_ttl: 120,
    };
    unresolved.sort_unstable();
    unresolved.dedup();
    Ok(
        serde_json::json!({"settings":{"saml":settings},"sp_encryption_certificate_pem":encryption,"unresolved":unresolved,"metadata_trust":"explicit_local_operator_input","required_scopes":if settings.name_id_format==NameIdFormat::Email{vec!["openid","saml","email"]}else{vec!["openid","saml"]},"instruction":"Review endpoints, NameID and attribute mappings; select a matching IdP signing domain, then apply through the normal client manifest."}),
    )
}
