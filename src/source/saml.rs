//! SAML source: signed SP requests, pinned responses, explicit links and terminal completion.
use super::{Login, Source, UpstreamIdentity};
use crate::{
    core::{Core, audit, validate_display, validate_email, validate_name},
    crypto::{self, Keys, SigningKey, digest, now},
    error::{Error, Result},
    jose::ClientAuthMethod,
    response::escape,
    saml::{
        NameIdFormat,
        wire::{self, ASSERTION, DSIG, METADATA, POST, PROTOCOL},
    },
    store::Tx,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use roxmltree::{Document, Node};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub signing_key: String,
    pub sp_certificate_pem: String,
    pub idp_certificates_pem: Vec<String>,
    #[serde(default)]
    pub name_id_format: NameIdFormat,
    pub name_attribute: Option<String>,
    pub email_attribute: Option<String>,
    pub email_verified_attribute: Option<String>,
    #[serde(default)]
    pub require_encrypted_assertions: bool,
    pub slo_redirect_url: Option<String>,
    pub slo_post_url: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct UpstreamSession {
    #[serde(default)]
    pub subject: Option<String>,
    pub index: String,
    pub expires_at: Option<u64>,
}
impl Settings {
    pub fn validate(&self, source: &Source) -> Result<()> {
        crate::saml::entity_id(&source.issuer)?;
        crate::saml::entity_id(&source.client_id)?;
        crate::saml::endpoint_url(&source.authorization_endpoint)?;
        validate_name(&self.signing_key)?;
        for endpoint in self.slo_redirect_url.iter().chain(&self.slo_post_url) {
            crate::saml::endpoint_url(endpoint)?;
        }
        if source.oauth_profile.is_some()
            || !source.token_endpoint.is_empty()
            || source.token_endpoint_auth_method != ClientAuthMethod::None
            || !source.jwks.keys.is_empty()
            || !source.scopes.is_empty()
        {
            return Err(Error::bad(
                "SAML sources use issuer/client entity IDs, an SSO authorization endpoint, none client authentication, empty scopes/token endpoint/JWKS and no OAuth profile",
            ));
        }
        if self.name_id_format == NameIdFormat::Transient
            || self.idp_certificates_pem.is_empty()
            || self.idp_certificates_pem.len() > 4
            || source.groups.len() > 64
            || source.trusted_mfa_acr.len() > 16
        {
            return Err(Error::bad(
                "SAML sources require stable NameIDs, 1..4 pinned certificates and bounded groups/ACRs",
            ));
        }
        for group in &source.groups {
            validate_name(group)?;
        }
        for context in &source.trusted_mfa_acr {
            wire::bounded_text(context, 256)?;
        }
        for cert in std::iter::once(&self.sp_certificate_pem).chain(&self.idp_certificates_pem) {
            wire::certificate(cert)?;
        }
        for name in self
            .name_attribute
            .iter()
            .chain(self.email_attribute.iter())
            .chain(self.email_verified_attribute.iter())
        {
            wire::bounded_text(name, 1024)?;
        }
        if self.email_verified_attribute.is_some() && self.email_attribute.is_none() {
            return Err(Error::bad(
                "SAML verified-email mapping requires an email mapping",
            ));
        }
        Ok(())
    }
    pub(crate) fn key(&self, tx: &Tx<'_>) -> Result<SigningKey> {
        let keys = if self.signing_key == "signing" {
            crate::core::keys(tx)?
        } else {
            tx.get::<Keys>("key_domains", &self.signing_key)?
                .ok_or_else(|| Error::bad("SAML source signing domain is missing"))?
        };
        let key = keys.active;
        if key.remote.is_some() || key.algorithm != "RS256" {
            return Err(Error::bad(
                "SAML source requires a local RS256 signing/decryption key",
            ));
        }
        let private =
            risaml::crypto::keys::load_private_key(&key.pem, None).map_err(Error::internal)?;
        let public = risaml::crypto::keys::load_certificate(&self.sp_certificate_pem)
            .map_err(Error::internal)?;
        if private.to_spki_der().is_none() || private.to_spki_der() != public.to_spki_der() {
            return Err(Error::bad(
                "SAML source SP certificate does not match its signing domain",
            ));
        }
        Ok(key)
    }
    pub(super) fn authorization(
        &self,
        tx: &Tx<'_>,
        core: &Core,
        source: &Source,
        pending: &Login,
        state: &str,
    ) -> Result<String> {
        let xml = format!(
            r#"<samlp:AuthnRequest xmlns:samlp="{PROTOCOL}" xmlns:saml="{ASSERTION}" ID="_{}" Version="2.0" IssueInstant="{}" Destination="{}" AssertionConsumerServiceURL="{}" ProtocolBinding="{POST}" ForceAuthn="true"><saml:Issuer>{}</saml:Issuer><samlp:NameIDPolicy Format="{}" AllowCreate="true"/></samlp:AuthnRequest>"#,
            pending.nonce,
            wire::timestamp(now())?,
            escape(&source.authorization_endpoint),
            escape(&core.saml_source_callback_url(&source.id)),
            escape(&source.client_id),
            self.name_id_format.uri()
        );
        wire::redirect_message(
            &source.authorization_endpoint,
            &xml,
            Some(state),
            &self.key(tx)?.pem,
            "SAMLRequest",
        )
    }
}
impl Core {
    pub fn saml_source_callback_url(&self, id: &str) -> String {
        format!(
            "{}/saml/sources/{id}/acs",
            crate::saml::endpoint_base(&self.config.issuer)
        )
    }
    pub fn saml_source_metadata(&self, id: &str) -> Result<String> {
        self.store.read(|tx|{
        let source=super::enabled(tx,id)?;let settings=source.saml.as_ref().ok_or_else(||Error::missing("SAML source not found"))?;settings.validate(&source)?;
        let cert=STANDARD.encode(wire::certificate(&settings.sp_certificate_pem)?);
        let slo=if settings.slo_redirect_url.is_some()||settings.slo_post_url.is_some() {format!(r#"<md:SingleLogoutService Binding="{}" Location="{}"/><md:SingleLogoutService Binding="{POST}" Location="{}"/>"#,wire::REDIRECT,escape(&format!("{}/saml/sources/{id}/slo",crate::saml::endpoint_base(&self.config.issuer))),escape(&format!("{}/saml/sources/{id}/slo",crate::saml::endpoint_base(&self.config.issuer))))}else{String::new()};
        let xml=format!(r#"<md:EntityDescriptor xmlns:md="{METADATA}" xmlns:ds="{DSIG}" ID="_{}" entityID="{}"><md:SPSSODescriptor protocolSupportEnumeration="{PROTOCOL}" AuthnRequestsSigned="true" WantAssertionsSigned="true"><md:KeyDescriptor use="signing"><ds:KeyInfo><ds:X509Data><ds:X509Certificate>{cert}</ds:X509Certificate></ds:X509Data></ds:KeyInfo></md:KeyDescriptor><md:KeyDescriptor use="encryption"><ds:KeyInfo><ds:X509Data><ds:X509Certificate>{cert}</ds:X509Certificate></ds:X509Data></ds:KeyInfo></md:KeyDescriptor>{slo}<md:NameIDFormat>{}</md:NameIDFormat><md:AssertionConsumerService Binding="{POST}" Location="{}" index="0" isDefault="true"/></md:SPSSODescriptor></md:EntityDescriptor>"#,crypto::id(),escape(&source.client_id),settings.name_id_format.uri(),escape(&self.saml_source_callback_url(id)));
        crate::saml::sign(&xml,&settings.key(tx)?,&settings.sp_certificate_pem,true)
    })
    }
    pub fn saml_source_callback(&self, id: &str, pairs: Vec<(String, String)>) -> Result<Value> {
        let mut params = BTreeMap::new();
        for (k, v) in pairs {
            if !["SAMLResponse", "RelayState"].contains(&k.as_str())
                || v.len() > 65536
                || params.insert(k, v).is_some()
            {
                return Err(Error::bad("Invalid SAML source POST fields"));
            }
        }
        let state = params
            .get("RelayState")
            .filter(|s| s.len() == 43)
            .ok_or_else(|| Error::bad("SAML source requires its original RelayState"))?;
        let encoded = params
            .get("SAMLResponse")
            .ok_or_else(|| Error::bad("Missing SAMLResponse"))?;
        let (source, pending) = self.store.write(|tx| {
            let source = super::enabled(tx, id)?;
            let settings = source
                .saml
                .as_ref()
                .ok_or_else(|| Error::bad("Source is not SAML"))?;
            settings.validate(&source)?;
            let mut pending = tx
                .get::<Login>("source_logins", &digest(state))?
                .filter(|p| p.source == id && !p.claimed && p.expires_at > now())
                .ok_or_else(|| Error::bad("SAML source request expired or already used"))?;
            if pending.fingerprint != source.fingerprint()? {
                return Err(Error::bad("SAML source changed; restart login"));
            }
            pending.claimed = true;
            tx.put("source_logins", &digest(state), &pending)?;
            Ok((source, pending))
        })?;
        let result = (|| {
            let xml = String::from_utf8(
                STANDARD
                    .decode(encoded)
                    .map_err(|_| Error::bad("Invalid SAMLResponse base64"))?,
            )
            .map_err(|_| Error::bad("SAML response must be UTF-8"))?;
            let settings = source.saml.as_ref().unwrap();
            let key = self.store.read(|tx| settings.key(tx))?;
            verified_identity(
                &xml,
                settings,
                &source,
                &pending,
                &self.saml_source_callback_url(id),
                &key,
            )
        })();
        self.store.write(|tx| {
            let current_source = super::enabled(tx, id)?;
            let mut current = tx
                .get::<Login>("source_logins", &digest(state))?
                .filter(|p| p.claimed && p.expires_at > now() && p.result.is_none() && !p.failed)
                .ok_or_else(|| Error::bad("SAML source request expired"))?;
            let result = result.and_then(|(identity, assertion, expiry)| {
                if current_source.fingerprint()? != pending.fingerprint {
                    return Err(Error::forbidden());
                }
                let key = digest(&format!("{}\0{assertion}", source.issuer));
                if tx
                    .get::<u64>("saml_source_replays", &key)?
                    .is_some_and(|at| at > now())
                {
                    return Err(Error::forbidden());
                }
                tx.put("saml_source_replays", &key, &expiry.saturating_add(30))?;
                Ok(identity)
            });
            match result {
                Ok(identity) => current.result = Some(identity),
                Err(_) => current.failed = true,
            };
            tx.put("source_logins", &digest(state), &current)?;
            audit(
                tx,
                "upstream",
                if current.failed {
                    "source.login_failed"
                } else {
                    "source.authenticated"
                },
                id,
            )?;
            super::callback_body(tx, &current, &digest(state))
        })
    }
}
fn limits() -> risaml::xml::XmlLimits {
    risaml::xml::XmlLimits {
        max_bytes: 48 * 1024,
        max_depth: 24,
        max_nodes: 2048,
        max_attributes_per_element: 32,
        max_attribute_value_bytes: 8192,
        max_text_bytes: 48 * 1024,
    }
}
fn required<'a, 'i>(node: Node<'a, 'i>, ns: &str, name: &str) -> Result<Node<'a, 'i>> {
    wire::child(node, ns, name)?.ok_or_else(|| Error::bad("Required SAML element is missing"))
}
fn children(node: Node<'_, '_>, allowed: &[(&str, &str)]) -> Result<()> {
    if node
        .children()
        .filter(Node::is_element)
        .any(|n| !allowed.iter().any(|tag| n.has_tag_name(*tag)))
    {
        return Err(Error::bad("Unsupported SAML response structure"));
    }
    Ok(())
}
fn id(node: Node<'_, '_>) -> Result<String> {
    let id = node
        .attribute("ID")
        .filter(|id| {
            !id.is_empty()
                && id.len() <= 128
                && id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_-.".contains(&b))
                && (id.as_bytes()[0].is_ascii_alphabetic() || id.as_bytes()[0] == b'_')
        })
        .ok_or_else(|| Error::bad("Invalid SAML response ID"))?;
    Ok(id.into())
}
fn signatures(doc: &Document<'_>) -> Result<()> {
    let mut ids = BTreeSet::new();
    for node in doc.descendants().filter(Node::is_element) {
        for attr in node
            .attributes()
            .filter(|a| ["ID", "Id", "id", "AssertionID"].contains(&a.name()))
        {
            if attr.namespace().is_some() || !ids.insert(attr.value()) {
                return Err(Error::bad("Duplicate or namespaced XML identifier"));
            }
        }
    }
    let root = doc.root_element();
    let mut count = 0;
    for signature in doc
        .descendants()
        .filter(|n| n.has_tag_name((DSIG, "Signature")))
    {
        count += 1;
        let owner = signature.parent().ok_or_else(Error::forbidden)?;
        if count > 2
            || !(owner == root
                || owner.parent() == Some(root) && owner.has_tag_name((ASSERTION, "Assertion")))
        {
            return Err(Error::bad(
                "Signature must be attached to the response or its one assertion",
            ));
        }
        let owner_id = id(owner)?;
        let refs = signature
            .descendants()
            .filter(|n| n.has_tag_name((DSIG, "Reference")))
            .collect::<Vec<_>>();
        if refs.len() != 1 || refs[0].attribute("URI") != Some(format!("#{owner_id}").as_str()) {
            return Err(Error::bad("SAML signature must cover its complete parent"));
        }
        let transforms = refs[0]
            .descendants()
            .filter(|n| n.has_tag_name((DSIG, "Transform")))
            .map(|n| n.attribute("Algorithm"))
            .collect::<Vec<_>>();
        if transforms != [Some(wire::ENV), Some(wire::EXC)] {
            return Err(Error::bad("Unsupported SAML signature transforms"));
        }
        for node in signature.descendants().filter(Node::is_element) {
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
            let expected = match node.tag_name().name() {
                "CanonicalizationMethod" => Some(wire::EXC),
                "SignatureMethod" => Some(wire::RSA256),
                "DigestMethod" => Some(wire::SHA256),
                _ => None,
            };
            if expected.is_some_and(|e| node.attribute("Algorithm") != Some(e)) {
                return Err(Error::bad(
                    "SAML source requires RSA-SHA256, SHA-256 and exclusive canonicalization",
                ));
            }
        }
    }
    Ok(())
}
fn assertion_signature(xml: &str, node: Node<'_, '_>, settings: &Settings) -> Result<()> {
    // Preserve the exact signed bytes and inherited namespace context. No claims come from this wrapper.
    required(node, DSIG, "Signature")?;
    let prefix = format!("ri_{}", crypto::id().replace('-', "_"));
    let namespaces = node
        .namespaces()
        .map(|ns| match ns.name() {
            Some(name) => format!(" xmlns:{name}=\"{}\"", escape(ns.uri())),
            None => format!(" xmlns=\"{}\"", escape(ns.uri())),
        })
        .collect::<String>();
    let assertion = &xml[node.range()];
    let wrapped = format!(
        "<{prefix}:Response xmlns:{prefix}=\"{PROTOCOL}\"{namespaces}>{assertion}</{prefix}:Response>"
    );
    let (valid, covered) = risaml::crypto::verify_signature_with_limits(
        &wrapped,
        &settings.idp_certificates_pem,
        limits(),
    )
    .map_err(|_| Error::forbidden())?;
    if !valid || covered.as_deref() != Some(assertion) {
        return Err(Error::forbidden());
    }
    Ok(())
}
const XENC: &str = "http://www.w3.org/2001/04/xmlenc#";
fn encrypted_structure(node: Node<'_, '_>) -> Result<()> {
    children(node, &[(XENC, "EncryptedData")])?;
    let data = required(node, XENC, "EncryptedData")?;
    let key = required(required(data, DSIG, "KeyInfo")?, XENC, "EncryptedKey")?;
    if required(data, XENC, "EncryptionMethod")?.attribute("Algorithm")
        != Some("http://www.w3.org/2009/xmlenc11#aes256-gcm")
        || required(key, XENC, "EncryptionMethod")?.attribute("Algorithm")
            != Some("http://www.w3.org/2001/04/xmlenc#rsa-oaep-mgf1p")
    {
        return Err(Error::bad(
            "SAML source encryption requires AES-256-GCM and RSA-OAEP",
        ));
    }
    for node in node.descendants().filter(Node::is_element).skip(1) {
        if ![
            (XENC, "EncryptedData"),
            (XENC, "EncryptionMethod"),
            (XENC, "EncryptedKey"),
            (XENC, "CipherData"),
            (XENC, "CipherValue"),
            (DSIG, "KeyInfo"),
            (DSIG, "KeyName"),
            (DSIG, "X509Data"),
            (DSIG, "X509Certificate"),
            (DSIG, "DigestMethod"),
        ]
        .iter()
        .any(|tag| node.has_tag_name(*tag))
        {
            return Err(Error::bad(
                "Unsupported XML encryption structure or reference",
            ));
        }
    }
    if data
        .descendants()
        .filter(|n| n.has_tag_name((XENC, "EncryptedData")))
        .count()
        != 1
        || data
            .descendants()
            .filter(|n| n.has_tag_name((XENC, "EncryptedKey")))
            .count()
            != 1
        || data
            .descendants()
            .filter(|n| n.has_tag_name((XENC, "CipherValue")))
            .count()
            != 2
    {
        return Err(Error::bad(
            "XML encryption requires one embedded ciphertext and encrypted key",
        ));
    }
    Ok(())
}
fn verified_identity(
    xml: &str,
    settings: &Settings,
    source: &Source,
    pending: &Login,
    acs: &str,
    key: &SigningKey,
) -> Result<(UpstreamIdentity, String, u64)> {
    let doc = wire::document(xml)?;
    let root = doc.root_element();
    if !root.has_tag_name((PROTOCOL, "Response")) {
        return Err(Error::bad("Expected SAML Response"));
    }
    wire::attributes(
        root,
        &[
            "ID",
            "Version",
            "IssueInstant",
            "Destination",
            "InResponseTo",
            "Consent",
        ],
    )?;
    id(root)?;
    let request_id = format!("_{}", pending.nonce);
    if root.attribute("Version") != Some("2.0")
        || root.attribute("Destination") != Some(acs)
        || root.attribute("InResponseTo") != Some(request_id.as_str())
        || wire::text(required(root, ASSERTION, "Issuer")?)? != source.issuer
    {
        return Err(Error::forbidden());
    }
    recent(
        root.attribute("IssueInstant")
            .ok_or_else(|| Error::bad("Missing SAML IssueInstant"))?,
        pending,
    )?;
    children(
        root,
        &[
            (ASSERTION, "Issuer"),
            (DSIG, "Signature"),
            (PROTOCOL, "Status"),
            (ASSERTION, "Assertion"),
            (ASSERTION, "EncryptedAssertion"),
        ],
    )?;
    let status = required(root, PROTOCOL, "Status")?;
    children(
        status,
        &[(PROTOCOL, "StatusCode"), (PROTOCOL, "StatusMessage")],
    )?;
    let status = required(status, PROTOCOL, "StatusCode")?;
    if status.attribute("Value") != Some("urn:oasis:names:tc:SAML:2.0:status:Success")
        || status.children().any(|n| n.is_element())
    {
        return Err(Error::forbidden());
    }
    signatures(&doc)?;
    let plain = wire::child(root, ASSERTION, "Assertion")?;
    let encrypted = wire::child(root, ASSERTION, "EncryptedAssertion")?;
    let response_signature = required(root, DSIG, "Signature")?;
    // risaml/ribergshamra verifies the first signature in document order. Require that
    // signature to be the response's own; verify the assertion independently below.
    if doc
        .descendants()
        .find(|n| n.has_tag_name((DSIG, "Signature")))
        != Some(response_signature)
    {
        return Err(Error::bad(
            "The response signature must precede its assertion",
        ));
    }
    let (valid, covered) =
        risaml::crypto::verify_signature_with_limits(xml, &settings.idp_certificates_pem, limits())
            .map_err(|_| Error::forbidden())?;
    let expected = plain.map(|a| &xml[a.range()]).unwrap_or(&xml[root.range()]);
    if !valid || covered.as_deref() != Some(expected) {
        return Err(Error::forbidden());
    }
    match (plain, encrypted) {
        (Some(assertion), None) if !settings.require_encrypted_assertions => {
            assertion_signature(xml, assertion, settings)?;
            identity(assertion, source, pending, acs, settings)
        }
        (None, Some(encrypted)) => {
            encrypted_structure(encrypted)?;
            let key =
                risaml::crypto::keys::load_private_key(&key.pem, None).map_err(Error::internal)?;
            let (_, assertion) = risaml::crypto::decrypt_assertion_with_limits(
                xml,
                &key,
                Default::default(),
                limits(),
            )
            .map_err(|_| Error::forbidden())?;
            let doc = wire::document(&assertion)?;
            let root = doc.root_element();
            if !root.has_tag_name((ASSERTION, "Assertion")) {
                return Err(Error::forbidden());
            }
            signatures(&doc)?;
            assertion_signature(&assertion, root, settings)?;
            identity(root, source, pending, acs, settings)
        }
        _ => Err(Error::bad(
            "SAML source requires exactly one signed assertion in its configured encryption profile",
        )),
    }
}
fn recent(value: &str, pending: &Login) -> Result<u64> {
    let at = wire::parse_time(value)?;
    if at > now() + 30 || at + 300 < now() || at + 30 < pending.started_at {
        return Err(Error::forbidden());
    }
    Ok(at)
}

fn identity(
    assertion: Node<'_, '_>,
    source: &Source,
    pending: &Login,
    acs: &str,
    settings: &Settings,
) -> Result<(UpstreamIdentity, String, u64)> {
    wire::attributes(assertion, &["ID", "Version", "IssueInstant"])?;
    let assertion_id = id(assertion)?;
    if assertion.attribute("Version") != Some("2.0") {
        return Err(Error::forbidden());
    }
    recent(
        assertion
            .attribute("IssueInstant")
            .ok_or_else(|| Error::bad("Missing assertion IssueInstant"))?,
        pending,
    )?;
    children(
        assertion,
        &[
            (ASSERTION, "Issuer"),
            (DSIG, "Signature"),
            (ASSERTION, "Subject"),
            (ASSERTION, "Conditions"),
            (ASSERTION, "AuthnStatement"),
            (ASSERTION, "AttributeStatement"),
        ],
    )?;
    if wire::text(required(assertion, ASSERTION, "Issuer")?)? != source.issuer {
        return Err(Error::forbidden());
    }
    let subject = required(assertion, ASSERTION, "Subject")?;
    wire::attributes(subject, &[])?;
    children(
        subject,
        &[(ASSERTION, "NameID"), (ASSERTION, "SubjectConfirmation")],
    )?;
    let name_id = required(subject, ASSERTION, "NameID")?;
    wire::attributes(name_id, &["Format", "NameQualifier", "SPNameQualifier"])?;
    if name_id
        .attribute("Format")
        .unwrap_or(NameIdFormat::Unspecified.uri())
        != settings.name_id_format.uri()
        || name_id
            .attribute("NameQualifier")
            .is_some_and(|v| v != source.issuer)
        || name_id
            .attribute("SPNameQualifier")
            .is_some_and(|v| v != source.client_id)
    {
        return Err(Error::forbidden());
    }
    let subject_name = wire::text(name_id)?;
    wire::bounded_text(&subject_name, 255)?;
    let confirmation = required(subject, ASSERTION, "SubjectConfirmation")?;
    wire::attributes(confirmation, &["Method"])?;
    children(confirmation, &[(ASSERTION, "SubjectConfirmationData")])?;
    if confirmation.attribute("Method") != Some("urn:oasis:names:tc:SAML:2.0:cm:bearer") {
        return Err(Error::forbidden());
    }
    let data = required(confirmation, ASSERTION, "SubjectConfirmationData")?;
    wire::attributes(data, &["Recipient", "NotOnOrAfter", "InResponseTo"])?;
    children(data, &[])?;
    let request_id = format!("_{}", pending.nonce);
    if data.attribute("Recipient") != Some(acs)
        || data.attribute("InResponseTo") != Some(&request_id)
    {
        return Err(Error::forbidden());
    }
    let confirmation_expiry = expiry(data)?;
    let conditions = required(assertion, ASSERTION, "Conditions")?;
    wire::attributes(conditions, &["NotBefore", "NotOnOrAfter"])?;
    children(
        conditions,
        &[
            (ASSERTION, "AudienceRestriction"),
            (ASSERTION, "OneTimeUse"),
        ],
    )?;
    if let Some(at) = conditions.attribute("NotBefore")
        && wire::parse_time(at)? > now() + 30
    {
        return Err(Error::forbidden());
    }
    let expires_at = expiry(conditions)?.min(confirmation_expiry);
    let audiences = conditions
        .children()
        .filter(|n| n.has_tag_name((ASSERTION, "AudienceRestriction")))
        .collect::<Vec<_>>();
    if audiences.is_empty() || audiences.len() > 8 {
        return Err(Error::forbidden());
    }
    for restriction in audiences {
        wire::attributes(restriction, &[])?;
        children(restriction, &[(ASSERTION, "Audience")])?;
        let values = restriction
            .children()
            .filter(Node::is_element)
            .map(wire::text)
            .collect::<Result<Vec<_>>>()?;
        if values.len() > 8 || !values.contains(&source.client_id) {
            return Err(Error::forbidden());
        }
    }
    if let Some(once) = wire::child(conditions, ASSERTION, "OneTimeUse")? {
        wire::attributes(once, &[])?;
        children(once, &[])?;
    }
    let auth = required(assertion, ASSERTION, "AuthnStatement")?;
    wire::attributes(
        auth,
        &["AuthnInstant", "SessionIndex", "SessionNotOnOrAfter"],
    )?;
    children(auth, &[(ASSERTION, "AuthnContext")])?;
    let auth_time = wire::parse_time(
        auth.attribute("AuthnInstant")
            .ok_or_else(|| Error::bad("Missing AuthnInstant"))?,
    )?;
    // Every SP request sets ForceAuthn. Source completion cannot turn an old login into a fresh proof.
    if auth_time + 5 < pending.started_at || auth_time > now() + 30 {
        return Err(Error::forbidden());
    }
    let index = auth
        .attribute("SessionIndex")
        .ok_or_else(|| Error::bad("SAML source requires SessionIndex"))?;
    wire::bounded_text(index, 256)?;
    let session_expiry = auth
        .attribute("SessionNotOnOrAfter")
        .map(wire::parse_time)
        .transpose()?;
    if session_expiry.is_some_and(|at| at <= now()) {
        return Err(Error::forbidden());
    }
    let context = required(auth, ASSERTION, "AuthnContext")?;
    children(context, &[(ASSERTION, "AuthnContextClassRef")])?;
    let context = wire::text(required(context, ASSERTION, "AuthnContextClassRef")?)?;
    let mut attributes = BTreeMap::new();
    if let Some(statement) = wire::child(assertion, ASSERTION, "AttributeStatement")? {
        wire::attributes(statement, &[])?;
        children(statement, &[(ASSERTION, "Attribute")])?;
        for attribute in statement.children().filter(Node::is_element) {
            wire::attributes(attribute, &["Name", "NameFormat", "FriendlyName"])?;
            children(attribute, &[(ASSERTION, "AttributeValue")])?;
            let name = attribute
                .attribute("Name")
                .ok_or_else(|| Error::bad("SAML Attribute lacks Name"))?;
            wire::bounded_text(name, 1024)?;
            let values = attribute
                .children()
                .filter(Node::is_element)
                .map(wire::text)
                .collect::<Result<Vec<_>>>()?;
            if attributes.len() >= 32
                || values.len() > 32
                || attributes.insert(name.to_owned(), values).is_some()
            {
                return Err(Error::bad("Duplicate or excessive SAML attributes"));
            }
        }
    }
    let read = |name: &Option<String>| -> Result<Option<String>> {
        match name.as_ref().and_then(|n| attributes.get(n)) {
            None => Ok(None),
            Some(values) if values.len() == 1 => Ok(Some(values[0].clone())),
            _ => Err(Error::bad(
                "Mapped SAML identity attributes require one value",
            )),
        }
    };
    let name = read(&settings.name_attribute)?.unwrap_or_else(|| "Federated user".into());
    validate_display(&name)?;
    let email = read(&settings.email_attribute)?;
    if let Some(email) = &email {
        validate_email(email)?;
    }
    let verified = read(&settings.email_verified_attribute)?;
    let email_verified = match verified.as_deref() {
        None | Some("false" | "0") => false,
        Some("true" | "1") if email.is_some() => true,
        _ => return Err(Error::bad("Invalid mapped SAML email verification value")),
    };
    Ok((
        UpstreamIdentity {
            subject: subject_name.clone(),
            name,
            email,
            email_verified,
            mfa: source.trusted_mfa_acr.contains(&context),
            auth_time,
            saml_session: Some(UpstreamSession {
                subject: Some(subject_name),
                index: index.into(),
                expires_at: session_expiry,
            }),
        },
        assertion_id,
        expires_at,
    ))
}
fn expiry(node: Node<'_, '_>) -> Result<u64> {
    let at = wire::parse_time(
        node.attribute("NotOnOrAfter")
            .ok_or_else(|| Error::bad("SAML bearer assertion requires an expiry"))?,
    )?;
    if at <= now() || at > now() + 600 {
        return Err(Error::forbidden());
    }
    Ok(at)
}
pub(super) fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (id, expiry) in tx.maintenance_page::<u64>("saml_source_replays")? {
        if expiry <= at {
            tx.delete("saml_source_replays", &id)?;
        }
    }
    for (id, _) in tx.maintenance_page::<UpstreamSession>("saml_source_sessions")? {
        if tx
            .get::<crate::model::Session>("sessions", &id)?
            .is_none_or(|s| s.expires_at <= at)
        {
            tx.delete("saml_source_sessions", &id)?;
        }
    }
    Ok(())
}
