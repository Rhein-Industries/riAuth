use super::*;
use base64::engine::general_purpose::STANDARD;
use openssl::{
    hash::MessageDigest,
    pkey::{PKey, Private},
    rsa::Rsa,
    sign::Verifier,
};
use riauth::{
    saml::NameIdFormat,
    source::saml::Settings,
    source::{Finish, Source, SourceInput, Start},
};
use std::{io::Read, path::Path};
const A: &str = "urn:oasis:names:tc:SAML:2.0:assertion";
const P: &str = "urn:oasis:names:tc:SAML:2.0:protocol";
const D: &str = "http://www.w3.org/2000/09/xmldsig#";
const ALG: &str = "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256";
fn at(value: u64) -> String {
    time::OffsetDateTime::from_unix_timestamp(value as i64)
        .unwrap()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap()
}
pub(super) struct Upstream {
    pub(super) key: PKey<Private>,
    pub(super) cert: String,
    pub(super) xmlsec: Option<std::path::PathBuf>,
    pub(super) dir: TempDir,
}
impl Upstream {
    pub(super) fn sign(&self, xml: &str, whole: bool) -> String {
        if let Some(binary) = &self.xmlsec {
            let doc = roxmltree::Document::parse(xml).unwrap();
            let root = doc.root_element();
            let id = root.attribute("ID").unwrap();
            let issuer = root
                .children()
                .find(|n| n.has_tag_name((A, "Issuer")))
                .unwrap();
            let cert = STANDARD.encode(pem::parse(&self.cert).unwrap().contents());
            let signature = format!(
                r##"<ds:Signature xmlns:ds="{D}"><ds:SignedInfo><ds:CanonicalizationMethod Algorithm="http://www.w3.org/2001/10/xml-exc-c14n#"/><ds:SignatureMethod Algorithm="{ALG}"/><ds:Reference URI="#{id}"><ds:Transforms><ds:Transform Algorithm="http://www.w3.org/2000/09/xmldsig#enveloped-signature"/><ds:Transform Algorithm="http://www.w3.org/2001/10/xml-exc-c14n#"/></ds:Transforms><ds:DigestMethod Algorithm="http://www.w3.org/2001/04/xmlenc#sha256"/><ds:DigestValue/></ds:Reference></ds:SignedInfo><ds:SignatureValue/><ds:KeyInfo><ds:X509Data><ds:X509Certificate>{cert}</ds:X509Certificate></ds:X509Data></ds:KeyInfo></ds:Signature>"##
            );
            let input = format!(
                "{}{}{}",
                &xml[..issuer.range().end],
                signature,
                &xml[issuer.range().end..]
            );
            let file = self.dir.path().join(format!("input-{id}.xml"));
            let out = self.dir.path().join(format!("output-{id}.xml"));
            std::fs::write(&file, input).unwrap();
            saml_tests::xmlsec(
                binary,
                &[
                    "--sign",
                    "--lax-key-search",
                    "--privkey-pem",
                    self.dir.path().join("idp.key").to_str().unwrap(),
                    "--id-attr:ID",
                    "Assertion",
                    "--id-attr:ID",
                    "Response",
                    "--id-attr:ID",
                    "LogoutRequest",
                    "--id-attr:ID",
                    "LogoutResponse",
                    "--output",
                    out.to_str().unwrap(),
                    file.to_str().unwrap(),
                ],
            );
            let signed = std::fs::read_to_string(out).unwrap();
            let doc = roxmltree::Document::parse(&signed).unwrap();
            signed[doc.root_element().range()].to_owned()
        } else {
            let key = risaml::crypto::keys::load_private_key(
                &String::from_utf8(self.key.private_key_to_pem_pkcs8().unwrap()).unwrap(),
                None,
            )
            .unwrap();
            risaml::crypto::construct_saml_signature(xml, whole, &key, &self.cert, ALG, &[], None)
                .unwrap()
        }
    }
    pub(super) fn response(
        &self,
        source: &Source,
        acs: &str,
        request: &str,
        change: Option<(&str, &str)>,
        encrypted: bool,
    ) -> String {
        let mut assertion = format!(
            r#"<saml:Assertion xmlns:saml="{A}" ID="_{}" Version="2.0" IssueInstant="{}"><saml:Issuer>{}</saml:Issuer><saml:Subject><saml:NameID Format="{}" NameQualifier="{}" SPNameQualifier="{}">opaque-subject</saml:NameID><saml:SubjectConfirmation Method="urn:oasis:names:tc:SAML:2.0:cm:bearer"><saml:SubjectConfirmationData Recipient="{acs}" InResponseTo="{request}" NotOnOrAfter="{}"/></saml:SubjectConfirmation></saml:Subject><saml:Conditions NotBefore="{}" NotOnOrAfter="{}"><saml:AudienceRestriction><saml:Audience>{}</saml:Audience></saml:AudienceRestriction></saml:Conditions><saml:AuthnStatement AuthnInstant="{}" SessionIndex="upstream-session" SessionNotOnOrAfter="{}"><saml:AuthnContext><saml:AuthnContextClassRef>urn:example:password</saml:AuthnContextClassRef></saml:AuthnContext></saml:AuthnStatement><saml:AttributeStatement><saml:Attribute Name="display"><saml:AttributeValue>Upstream &amp; Partners</saml:AttributeValue></saml:Attribute><saml:Attribute Name="email"><saml:AttributeValue>unrelated@example.test</saml:AttributeValue></saml:Attribute><saml:Attribute Name="verified"><saml:AttributeValue>true</saml:AttributeValue></saml:Attribute></saml:AttributeStatement></saml:Assertion>"#,
            crypto::id(),
            at(now()),
            source.issuer,
            NameIdFormat::Persistent.uri(),
            source.issuer,
            source.client_id,
            at(now() + 120),
            at(now() - 1),
            at(now() + 120),
            source.client_id,
            at(now()),
            at(now() + 600)
        );
        if let Some((from, to)) = change {
            if from == "$stale-authn" {
                let doc = roxmltree::Document::parse(&assertion).unwrap();
                let auth = doc
                    .descendants()
                    .find_map(|n| n.attribute("AuthnInstant"))
                    .unwrap()
                    .to_owned();
                assertion = assertion.replace(
                    &format!(r#"AuthnInstant="{auth}""#),
                    &format!(r#"AuthnInstant="{}""#, at(now() - 600)),
                );
            } else if from == "$session-expiry" {
                let doc = roxmltree::Document::parse(&assertion).unwrap();
                let expiry = doc
                    .descendants()
                    .find_map(|n| n.attribute("SessionNotOnOrAfter"))
                    .unwrap();
                assertion = assertion.replace(
                    &format!(r#"SessionNotOnOrAfter="{expiry}""#),
                    &format!(r#"SessionNotOnOrAfter="{to}""#),
                );
            } else {
                assertion = assertion.replace(from, to);
            }
        }
        let mut assertion = self.sign(&assertion, false);
        if change.is_some_and(|(from, _)| from == "$inherit-ns") {
            assertion = assertion.replace(&format!(r#" xmlns:saml="{A}""#), "");
        }
        let mut response = format!(
            r#"<samlp:Response xmlns:samlp="{P}" xmlns:saml="{A}" ID="_{}" Version="2.0" IssueInstant="{}" Destination="{acs}" InResponseTo="{request}"><saml:Issuer>{}</saml:Issuer><samlp:Status><samlp:StatusCode Value="urn:oasis:names:tc:SAML:2.0:status:Success"/></samlp:Status>{assertion}</samlp:Response>"#,
            crypto::id(),
            at(now()),
            source.issuer
        );
        if encrypted {
            response = risaml::crypto::encrypt_assertion(
                &response,
                &source.saml.as_ref().unwrap().sp_certificate_pem,
                "http://www.w3.org/2009/xmlenc11#aes256-gcm",
                "http://www.w3.org/2001/04/xmlenc#rsa-oaep-mgf1p",
                "saml",
            )
            .unwrap();
        }
        self.sign(&response, true)
    }
}
fn decode_redirect(source: &Source, authorization_url: &str, acs: &str) -> (String, String) {
    let url = url::Url::parse(authorization_url).unwrap();
    let fields = url
        .query_pairs()
        .into_owned()
        .collect::<std::collections::BTreeMap<_, _>>();
    let (signed, encoded_signature) = url.query().unwrap().rsplit_once("&Signature=").unwrap();
    let signature = url::form_urlencoded::parse(format!("v={encoded_signature}").as_bytes())
        .next()
        .unwrap()
        .1
        .into_owned();
    let cert =
        openssl::x509::X509::from_pem(source.saml.as_ref().unwrap().sp_certificate_pem.as_bytes())
            .unwrap();
    let public = cert.public_key().unwrap();
    let mut verifier = Verifier::new(MessageDigest::sha256(), &public).unwrap();
    verifier.update(signed.as_bytes()).unwrap();
    assert!(
        verifier
            .verify(&STANDARD.decode(signature).unwrap())
            .unwrap()
    );
    let mut xml = String::new();
    flate2::read::DeflateDecoder::new(STANDARD.decode(&fields["SAMLRequest"]).unwrap().as_slice())
        .read_to_string(&mut xml)
        .unwrap();
    let doc = roxmltree::Document::parse(&xml).unwrap();
    let root = doc.root_element();
    assert_eq!(root.attribute("ForceAuthn"), Some("true"));
    assert_eq!(root.attribute("AssertionConsumerServiceURL"), Some(acs));
    (
        fields["RelayState"].clone(),
        root.attribute("ID").unwrap().into(),
    )
}
pub(super) fn begin(f: &Fixture, source: &Source, link: Option<&str>) -> (String, String, String) {
    let start = f
        .core
        .source_start(
            &source.id,
            Start {
                link: link.is_some(),
                authentication_transaction: None,
            },
            link,
        )
        .unwrap();
    let (relay, request) = decode_redirect(
        source,
        start["authorization_url"].as_str().unwrap(),
        &f.core.saml_source_callback_url(&source.id),
    );
    (relay, request, text(&start["credential"], "token"))
}
fn submit(f: &Fixture, source: &Source, state: &str, response: &str) -> Value {
    f.core
        .saml_source_callback(
            &source.id,
            vec![
                ("SAMLResponse".into(), STANDARD.encode(response)),
                ("RelayState".into(), state.into()),
            ],
        )
        .unwrap()
}
fn finish(f: &Fixture, credential: &str, approve: bool) -> riauth::error::Result<Value> {
    f.core.source_finish(Finish {
        credential: credential.into(),
        approve,
        otp: None,
    })
}

#[test]
fn saml_source_signed_encrypted_terminal_linking_and_live_trust() {
    exercise(None)
}
#[test]
#[ignore = "requires RIAUTH_TEST_XMLSEC"]
fn saml_source_independent_xmlsec_responses() {
    exercise(Some(std::env::var("RIAUTH_TEST_XMLSEC").unwrap().as_ref()))
}
fn exercise(xmlsec: Option<&Path>) {
    let f = Fixture::new();
    let alice = f.user("alice");
    f.user("unrelated");
    let key = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();
    let cert = saml_tests::cert(&key, "Upstream IdP");
    let upstream = Upstream {
        key,
        cert,
        xmlsec: xmlsec.map(Path::to_owned),
        dir: tempfile::tempdir().unwrap(),
    };
    riauth::config::write_private(
        &upstream.dir.path().join("idp.key"),
        &upstream.key.private_key_to_pem_pkcs8().unwrap(),
        false,
    )
    .unwrap();
    let keys: crypto::Keys = f.core.store.get("meta", "keys").unwrap().unwrap();
    let sp = PKey::private_key_from_pem(keys.active.pem.as_bytes()).unwrap();
    let sp_cert = saml_tests::cert(&sp, "riAuth source SP");
    let mut source = Source {
        saml: Some(Settings {
            slo_redirect_url: None,
            slo_post_url: None,
            signing_key: "signing".into(),
            sp_certificate_pem: sp_cert.clone(),
            idp_certificates_pem: vec![upstream.cert.clone()],
            name_id_format: NameIdFormat::Persistent,
            name_attribute: Some("display".into()),
            email_attribute: Some("email".into()),
            email_verified_attribute: Some("verified".into()),
            require_encrypted_assertions: false,
        }),
        oauth_profile: None,
        id: "enterprise".into(),
        name: "Enterprise SAML".into(),
        issuer: "urn:example:enterprise-idp".into(),
        authorization_endpoint: "https://enterprise.example.test/sso".into(),
        token_endpoint: "".into(),
        client_id: "urn:example:riauth-sp".into(),
        token_endpoint_auth_method: riauth::jose::ClientAuthMethod::None,
        jwks: Default::default(),
        scopes: Default::default(),
        enabled: true,
        auto_provision: false,
        groups: Default::default(),
        trusted_mfa_acr: Default::default(),
        allow_admin_login: false,
    };
    f.core
        .source_put(
            &f.admin,
            SourceInput {
                source: source.clone(),
                client_secret: None,
            },
        )
        .unwrap();
    let acs = f.core.saml_source_callback_url(&source.id);
    let metadata = f.core.saml_source_metadata(&source.id).unwrap();
    assert!(metadata.contains(&acs));
    assert!(metadata.contains(r#"AuthnRequestsSigned="true""#));
    if let Some(binary) = xmlsec {
        let file = f._dir.path().join("source-metadata.xml");
        let cert = f._dir.path().join("source-sp.pem");
        std::fs::write(&file, &metadata).unwrap();
        std::fs::write(&cert, &sp_cert).unwrap();
        saml_tests::xmlsec(
            binary,
            &[
                "--verify",
                "--trusted-pem",
                cert.to_str().unwrap(),
                "--id-attr:ID",
                "EntityDescriptor",
                file.to_str().unwrap(),
            ],
        );
    }
    let (state, id, credential) = begin(&f, &source, None);
    let response = upstream.response(&source, &acs, &id, None, false);
    assert_eq!(submit(&f, &source, &state, &response)["completed"], true);
    let review = finish(&f, &credential, false).unwrap();
    assert!(review["local_user"].is_null());
    assert_eq!(review["email"], "unrelated@example.test");
    assert!(finish(&f, &credential, true).is_err());
    let (state, id, credential) = begin(&f, &source, Some(&alice));
    let response = upstream.response(&source, &acs, &id, None, false);
    assert_eq!(submit(&f, &source, &state, &response)["completed"], true);
    let review = finish(&f, &credential, false).unwrap();
    assert_eq!(review["local_user"]["username"], "alice");
    assert_eq!(review["mfa"], false);
    let logged = finish(&f, &credential, true).unwrap();
    let token = text(&logged, "session_token");
    assert_eq!(f.core.me(&token).unwrap()["user"]["username"], "alice");
    assert!(finish(&f, &credential, true).is_err());

    // W03 consumes this same signed verifier output without minting a session.
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    let workflow = f.core.workflow_source_start(&alice, &source.id).unwrap();
    let url = url::Url::parse(&workflow.authorization_url).unwrap();
    let fields: std::collections::BTreeMap<_, _> = url.query_pairs().into_owned().collect();
    let mut request = String::new();
    flate2::read::DeflateDecoder::new(STANDARD.decode(&fields["SAMLRequest"]).unwrap().as_slice())
        .read_to_string(&mut request)
        .unwrap();
    let request = roxmltree::Document::parse(&request).unwrap();
    assert_eq!(request.root_element().attribute("ForceAuthn"), Some("true"));
    let session_expiry = now() + 60;
    let workflow_response = upstream.response(
        &source,
        &acs,
        request.root_element().attribute("ID").unwrap(),
        Some(("$session-expiry", &at(session_expiry))),
        false,
    );
    assert_eq!(
        submit(&f, &source, &fields["RelayState"], &workflow_response)["completed"],
        true
    );
    let login_key = digest(&fields["RelayState"]);
    let verified: Value = f
        .core
        .store
        .get("source_logins", &login_key)
        .unwrap()
        .unwrap();
    for (pointer, replacement) in [
        ("/result/expires_at", json!(now())),
        ("/result/saml_session/expires_at", json!(now())),
        ("/result/saml_session", Value::Null),
    ] {
        let mut expired = verified.clone();
        *expired.pointer_mut(pointer).unwrap() = replacement;
        f.core
            .store
            .write(|tx| tx.put("source_logins", &login_key, &expired))
            .unwrap();
        assert!(
            f.core
                .workflow_source_finish(&alice, &workflow.workflow.id)
                .is_err()
        );
        assert!(
            f.core
                .store
                .list::<Value>("workflow_evidence")
                .unwrap()
                .is_empty()
        );
        assert!(
            f.core
                .store
                .get::<Value>("source_logins", &login_key)
                .unwrap()
                .is_some()
        );
    }
    f.core
        .store
        .write(|tx| tx.put("source_logins", &login_key, &verified))
        .unwrap();
    let completed = f
        .core
        .workflow_source_finish(&alice, &workflow.workflow.id)
        .unwrap();
    assert!(matches!(
        completed.state,
        riauth::workflow::RunState::Finished {
            outcome: riauth::workflow::Outcome::Authenticated,
            ..
        }
    ));
    let receipts = f.core.store.list::<Value>("workflow_evidence").unwrap();
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].1["consumed"], true);
    assert!(receipts[0].1["expires_at"].as_u64().unwrap() <= session_expiry);
    assert_eq!(receipts[0].1["source"]["transaction"], login_key);
    assert!(
        f.core
            .store
            .get::<Value>("source_logins", &login_key)
            .unwrap()
            .is_none()
    );
    assert!(
        f.core
            .workflow_source_finish(&alice, &workflow.workflow.id)
            .is_err()
    );
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    assert!(
        f.core
            .saml_source_callback(
                &source.id,
                vec![
                    ("SAMLResponse".into(), STANDARD.encode(&response)),
                    ("RelayState".into(), state)
                ]
            )
            .is_err()
    );
    for change in [
        ("opaque-subject", ""),
        ("Recipient=", "WrongRecipient="),
        (
            "urn:example:riauth-sp</saml:Audience>",
            "urn:attacker</saml:Audience>",
        ),
        ("InResponseTo=", "OtherRequest="),
        (
            "urn:example:enterprise-idp</saml:Issuer>",
            "urn:attacker</saml:Issuer>",
        ),
    ] {
        let (state, id, credential) = begin(&f, &source, None);
        let response = upstream.response(&source, &acs, &id, Some(change), false);
        assert_eq!(
            submit(&f, &source, &state, &response)["completed"],
            false,
            "accepted {}",
            change.0
        );
        assert!(finish(&f, &credential, true).is_err());
    }
    let (state, id, credential) = begin(&f, &source, None);
    let response = upstream.response(&source, &acs, &id, Some(("$inherit-ns", "")), false);
    assert_eq!(submit(&f, &source, &state, &response)["completed"], true);
    assert_eq!(
        finish(&f, &credential, true).unwrap()["user"]["username"],
        "alice"
    );
    let (state, id, _) = begin(&f, &source, None);
    let response = upstream.response(&source, &acs, &id, Some(("$stale-authn", "")), false);
    assert_eq!(submit(&f, &source, &state, &response)["completed"], false);
    let (state, id, _) = begin(&f, &source, None);
    let response = upstream
        .response(&source, &acs, &id, None, false)
        .replace("Upstream &amp; Partners", "attacker");
    assert_eq!(submit(&f, &source, &state, &response)["completed"], false);
    let (state, id, _) = begin(&f, &source, None);
    let response = upstream.response(&source, &acs, &id, None, false);
    let (other, _, _) = begin(&f, &source, None);
    assert_eq!(submit(&f, &source, &other, &response)["completed"], false);
    assert_eq!(submit(&f, &source, &state, &response)["completed"], true);
    source.saml.as_mut().unwrap().require_encrypted_assertions = true;
    source.trusted_mfa_acr.insert("urn:example:mfa".into());
    f.core
        .source_put(
            &f.admin,
            SourceInput {
                source: source.clone(),
                client_secret: None,
            },
        )
        .unwrap();
    assert!(f.core.me(&token).is_err());
    let (state, id, _) = begin(&f, &source, None);
    let response = upstream.response(&source, &acs, &id, None, false);
    assert_eq!(submit(&f, &source, &state, &response)["completed"], false);
    let (state, id, credential) = begin(&f, &source, None);
    let response = upstream.response(
        &source,
        &acs,
        &id,
        Some(("urn:example:password", "urn:example:mfa")),
        true,
    );
    assert_eq!(submit(&f, &source, &state, &response)["completed"], true);
    assert_eq!(finish(&f, &credential, false).unwrap()["mfa"], true);
    let logged = finish(&f, &credential, true).unwrap();
    let token = text(&logged, "session_token");
    assert_eq!(f.core.me(&token).unwrap()["user"]["username"], "alice");
    // Drive the SAML ACS as an embedded stage, including encrypted assertions.
    // This also runs with independent xmlsec signatures in the opt-in fixture.
    f.client("stage-app", false);
    f.core
        .update_client(
            &f.admin,
            "stage-app",
            ClientPatch {
                settings: Some(ProviderSettings {
                    source_stage: Some(source.id.clone()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();
    let verifier = crypto::random_token("");
    let mut request = f.request("stage-app", &verifier);
    request.prompt = Some("login".into());
    request.decision = None;
    let prepared = f.core.authorization_prepare(Some(&alice), request).unwrap();
    let stage = &prepared["source_stage"];
    let url = url::Url::parse(stage["authorization_url"].as_str().unwrap()).unwrap();
    let fields: std::collections::BTreeMap<_, _> = url.query_pairs().into_owned().collect();
    let mut xml = String::new();
    flate2::read::DeflateDecoder::new(STANDARD.decode(&fields["SAMLRequest"]).unwrap().as_slice())
        .read_to_string(&mut xml)
        .unwrap();
    let doc = roxmltree::Document::parse(&xml).unwrap();
    let id = doc.root_element().attribute("ID").unwrap();
    let response = upstream.response(
        &source,
        &acs,
        id,
        Some(("urn:example:password", "urn:example:mfa")),
        true,
    );
    let callback = submit(&f, &source, &fields["RelayState"], &response);
    assert_eq!(callback["completed"], true);
    assert_eq!(callback["source_stage"]["stage_id"], stage["stage_id"]);
    assert!(callback.get("session_token").is_none());
    assert!(callback.get("code").is_none());
    let resumed = f
        .core
        .source_stage_resume(
            stage["stage_id"].as_str().unwrap(),
            stage["authorization_id"].as_str().unwrap(),
            None,
        )
        .unwrap();
    assert_eq!(resumed["code_issued"], true);
    let callback = url::Url::parse(resumed["redirect_uri"].as_str().unwrap()).unwrap();
    let fields: std::collections::BTreeMap<_, _> = callback.query_pairs().into_owned().collect();
    let grant: Code = f
        .core
        .store
        .get("codes", &digest(&fields["code"]))
        .unwrap()
        .unwrap();
    assert_eq!(
        grant.identity.user_id,
        f.core.me(&alice).unwrap()["user"]["id"]
    );
    assert!(grant.identity.mfa);
    assert_eq!(grant.challenge, digest(&verifier));
    assert_eq!(fields["state"], "state with & delimiters");
    assert!(
        f.core
            .source_stage_resume(
                stage["stage_id"].as_str().unwrap(),
                stage["authorization_id"].as_str().unwrap(),
                None
            )
            .is_err()
    );
    let links = f.core.source_links(&alice).unwrap();
    f.core
        .source_unlink(&alice, links[0]["id"].as_str().unwrap())
        .unwrap();
    assert!(f.core.me(&token).is_err());
}

fn install_saml(f: &Fixture) -> (Source, Upstream) {
    let key = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();
    let cert = saml_tests::cert(&key, "Upstream IdP");
    let upstream = Upstream {
        key,
        cert,
        xmlsec: None,
        dir: tempfile::tempdir().unwrap(),
    };
    riauth::config::write_private(
        &upstream.dir.path().join("idp.key"),
        &upstream.key.private_key_to_pem_pkcs8().unwrap(),
        false,
    )
    .unwrap();
    let keys: crypto::Keys = f.core.store.get("meta", "keys").unwrap().unwrap();
    let sp = PKey::private_key_from_pem(keys.active.pem.as_bytes()).unwrap();
    let source = Source {
        saml: Some(Settings {
            slo_redirect_url: None,
            slo_post_url: None,
            signing_key: "signing".into(),
            sp_certificate_pem: saml_tests::cert(&sp, "riAuth source SP"),
            idp_certificates_pem: vec![upstream.cert.clone()],
            name_id_format: NameIdFormat::Persistent,
            name_attribute: Some("display".into()),
            email_attribute: Some("email".into()),
            email_verified_attribute: Some("verified".into()),
            require_encrypted_assertions: false,
        }),
        oauth_profile: None,
        id: "enterprise".into(),
        name: "Enterprise SAML".into(),
        issuer: "urn:example:enterprise-idp".into(),
        authorization_endpoint: "https://enterprise.example.test/sso".into(),
        token_endpoint: "".into(),
        client_id: "urn:example:riauth-sp".into(),
        token_endpoint_auth_method: riauth::jose::ClientAuthMethod::None,
        jwks: Default::default(),
        scopes: Default::default(),
        enabled: true,
        auto_provision: true,
        groups: Default::default(),
        trusted_mfa_acr: Default::default(),
        allow_admin_login: false,
    };
    f.core
        .source_put(
            &f.admin,
            SourceInput {
                source: source.clone(),
                client_secret: None,
            },
        )
        .unwrap();
    (source, upstream)
}

fn login_row(f: &Fixture, relay: &str) -> Value {
    f.core
        .store
        .get::<Value>("source_logins", &digest(relay))
        .unwrap()
        .unwrap()
}

fn accounts(f: &Fixture) -> (usize, usize, usize) {
    (
        f.core
            .store
            .list::<riauth::model::User>("users")
            .unwrap()
            .len(),
        f.core
            .store
            .list::<riauth::model::Session>("sessions")
            .unwrap()
            .len(),
        f.core.store.list::<Value>("source_links").unwrap().len(),
    )
}

fn failed_audits(f: &Fixture) -> usize {
    f.core
        .store
        .list::<Value>("audit")
        .unwrap()
        .iter()
        .filter(|(_, event)| {
            event["action"] == "source.login_failed"
                && event["actor"] == "upstream"
                && event["target"] == "enterprise"
        })
        .count()
}

fn hides(error: &riauth::error::Error, secrets: &[&str]) {
    let shown = error.to_string();
    for secret in secrets {
        assert!(!shown.contains(secret), "{shown}");
    }
}

fn cookie_value(cookies: &[String], name: &str) -> String {
    let prefix = format!("{name}=");
    cookies
        .iter()
        .find_map(|cookie| {
            cookie
                .split(';')
                .next()
                .and_then(|pair| pair.strip_prefix(&prefix))
        })
        .unwrap_or_else(|| panic!("{name}"))
        .to_owned()
}

struct StartedBrowser {
    relay: String,
    request: String,
    cookie: String,
    credential: String,
}

fn browser_begin(f: &Fixture, source: &Source) -> StartedBrowser {
    let started = f.core.portal_source_start(None, &source.id, None).unwrap();
    let cookie = cookie_value(&started.cookies, "riauth_source");
    let (relay, request) = decode_redirect(
        source,
        started.body["authorization_url"].as_str().unwrap(),
        &f.core.saml_source_callback_url(&source.id),
    );
    let credential = cookie.split_once('.').unwrap().0.to_owned();
    let row = login_row(f, &relay);
    assert_eq!(
        row["browser_binding"],
        digest(&format!("{credential}.{}", digest(&relay)))
    );
    assert_eq!(row["browser_return_confirmed"], false);
    assert!(row["browser_return"].is_null());
    StartedBrowser {
        relay,
        request,
        cookie,
        credential,
    }
}

struct HttpReply {
    status: u16,
    location: String,
    set_cookie: Option<String>,
    referrer: String,
    body: String,
}

async fn read_reply(response: axum::response::Response) -> HttpReply {
    use http_body_util::BodyExt;
    let status = response.status().as_u16();
    let location = response
        .headers()
        .get("location")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_owned();
    let set_cookie = response
        .headers()
        .get("set-cookie")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let referrer = response
        .headers()
        .get("referrer-policy")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_owned();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    HttpReply {
        status,
        location,
        set_cookie,
        referrer,
        body: String::from_utf8_lossy(&bytes).into_owned(),
    }
}

async fn post_acs(core: &riauth::core::Core, id: &str, relay: &str, xml: &str) -> HttpReply {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;
    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("SAMLResponse", &STANDARD.encode(xml))
        .append_pair("RelayState", relay)
        .finish();
    let response = riauth::api::router(core.clone())
        .oneshot(
            Request::post(format!("/saml/sources/{id}/acs"))
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    read_reply(response).await
}

async fn get_return(core: &riauth::core::Core, id: &str, cookie: Option<&str>) -> HttpReply {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;
    let mut request = Request::get(format!("/saml/sources/{id}/return"))
        .body(Body::empty())
        .unwrap();
    if let Some(cookie) = cookie {
        request
            .headers_mut()
            .insert("cookie", axum::http::HeaderValue::from_str(cookie).unwrap());
    }
    read_reply(
        riauth::api::router(core.clone())
            .oneshot(request)
            .await
            .unwrap(),
    )
    .await
}

fn cookie_assignment(header: &str) -> (String, String) {
    let (name, value) = header.split(';').next().unwrap().split_once('=').unwrap();
    (name.to_owned(), value.to_owned())
}

fn relay_response(upstream: &Upstream, source: &Source, acs: &str, request: &str) -> String {
    upstream.response(source, acs, request, None, false)
}

/// Browser SAML cannot trust the Lax start cookie on the cross-site ACS POST.
/// Finish stays closed until a same-site return presents that cookie and the
/// one-time cookie set on the ACS response.
#[tokio::test]
async fn saml_browser_acs_handoff_checks_success_foreign_browser_missing_cookie_replay_and_cli() {
    let f = Fixture::new();
    let (source, upstream) = install_saml(&f);
    let acs = f.core.saml_source_callback_url(&source.id);
    let before = accounts(&f);
    let listed = f.core.portal_source_list().unwrap();
    assert!(
        listed["sources"]
            .as_array()
            .unwrap()
            .iter()
            .any(|source| source["id"] == "enterprise")
    );

    // CLI: the ACS body has no return token, and a return call does not end the login.
    let (relay, request, credential) = begin(&f, &source, None);
    let cli = post_acs(
        &f.core,
        &source.id,
        &relay,
        &relay_response(&upstream, &source, &acs, &request),
    )
    .await;
    assert_eq!(cli.status, 200);
    assert!(cli.set_cookie.is_none());
    let cli_body: Value = serde_json::from_str(&cli.body).unwrap();
    assert_eq!(cli_body["completed"], true);
    assert!(cli_body.get("browser_return").is_none());
    assert!(!cli.body.contains(&credential));
    assert!(!cli.body.contains(&relay));
    let cli_row = login_row(&f, &relay);
    assert!(cli_row["browser_binding"].is_null());
    assert!(cli_row["browser_return"].is_null());
    assert_eq!(cli_row["browser_return_confirmed"], true);
    assert_eq!(cli_row["result"]["subject"], "opaque-subject");
    assert_eq!(finish(&f, &credential, false).unwrap()["status"], "review");
    let unused = f
        .core
        .saml_source_browser_return(&source.id, None, None)
        .unwrap_err();
    assert_eq!(unused.status.as_u16(), 400);
    assert_eq!(unused.code, "invalid_request");
    let junk = f
        .core
        .saml_source_browser_return(&source.id, Some("not-a-cookie"), Some("not-a-return"))
        .unwrap_err();
    assert_eq!(junk.status.as_u16(), 400);
    hides(&junk, &[&credential, &relay]);
    assert_eq!(finish(&f, &credential, false).unwrap()["status"], "review");
    assert!(
        f.core
            .store
            .list::<String>("source_returns")
            .unwrap()
            .is_empty()
    );
    assert_eq!(accounts(&f), before);

    // A rejected assertion stores no return token and does not redirect.
    let rejected = browser_begin(&f, &source);
    let bad = upstream.response(
        &source,
        &acs,
        &rejected.request,
        Some((
            "urn:example:riauth-sp</saml:Audience>",
            "urn:attacker</saml:Audience>",
        )),
        false,
    );
    let rejected_acs = post_acs(&f.core, &source.id, &rejected.relay, &bad).await;
    assert_eq!(rejected_acs.status, 200);
    assert!(rejected_acs.set_cookie.is_none());
    let rejected_body: Value = serde_json::from_str(&rejected_acs.body).unwrap();
    assert_eq!(rejected_body["completed"], false);
    assert!(rejected_body.get("browser_return").is_none());
    assert!(!rejected_acs.body.contains(&rejected.cookie));
    assert!(!rejected_acs.body.contains(&rejected.relay));
    let rejected_row = login_row(&f, &rejected.relay);
    assert_eq!(rejected_row["failed"], true);
    assert!(rejected_row["result"].is_null());
    assert!(rejected_row["browser_return"].is_null());
    assert!(finish(&f, &rejected.credential, false).is_err());
    assert_eq!(accounts(&f), before);

    // Missing start cookie: the return token is real, so the login ends.
    let missing = browser_begin(&f, &source);
    let missing_audits = failed_audits(&f);
    let missing_acs = post_acs(
        &f.core,
        &source.id,
        &missing.relay,
        &relay_response(&upstream, &source, &acs, &missing.request),
    )
    .await;
    assert_eq!(missing_acs.status, 303);
    let (return_name, missing_token) =
        cookie_assignment(missing_acs.set_cookie.as_deref().unwrap());
    assert_eq!(return_name, "riauth_source_return");
    assert!(!missing_acs.body.contains(&missing_token));
    assert!(!missing_acs.location.contains(&missing_token));
    let burned = f
        .core
        .saml_source_browser_return(&source.id, None, Some(&missing_token))
        .unwrap_err();
    assert_eq!(burned.status.as_u16(), 403);
    assert_eq!(burned.code, "source_browser_mismatch");
    hides(&burned, &[&missing.cookie, &missing_token, &missing.relay]);
    let missing_row = login_row(&f, &missing.relay);
    assert_eq!(missing_row["failed"], true);
    assert!(missing_row["result"].is_null());
    assert!(missing_row["browser_return"].is_null());
    assert!(
        f.core
            .store
            .list::<String>("source_returns")
            .unwrap()
            .is_empty()
    );
    let too_late = f
        .core
        .saml_source_browser_return(&source.id, Some(&missing.cookie), Some(&missing_token))
        .unwrap_err();
    assert_eq!(too_late.status.as_u16(), 400);
    assert_ne!(too_late.code, "source_browser_mismatch");
    assert!(too_late.to_string().contains("already used"));
    hides(
        &too_late,
        &[&missing.cookie, &missing_token, &missing.relay],
    );
    assert!(finish(&f, &missing.credential, true).is_err());
    assert_eq!(
        f.core
            .portal_source_review(Some(&missing.credential))
            .unwrap_err()
            .code,
        "source_login_expired"
    );
    assert_eq!(failed_audits(&f), missing_audits + 1);
    assert_eq!(accounts(&f), before);

    // Foreign browser: B's start cookie with A's return token ends A and does not adopt it.
    let first = browser_begin(&f, &source);
    let second = browser_begin(&f, &source);
    let foreign_audits = failed_audits(&f);
    let first_acs = post_acs(
        &f.core,
        &source.id,
        &first.relay,
        &relay_response(&upstream, &source, &acs, &first.request),
    )
    .await;
    assert_eq!(first_acs.status, 303);
    let (_, first_token) = cookie_assignment(first_acs.set_cookie.as_deref().unwrap());
    assert!(finish(&f, &first.credential, false).is_err());
    assert_eq!(login_row(&f, &first.relay)["failed"], false);
    assert_eq!(
        login_row(&f, &first.relay)["result"]["subject"],
        "opaque-subject"
    );
    let foreign = f
        .core
        .saml_source_browser_return(&source.id, Some(&second.cookie), Some(&first_token))
        .unwrap_err();
    assert_eq!(foreign.status.as_u16(), 403);
    assert_eq!(foreign.code, "source_browser_mismatch");
    hides(
        &foreign,
        &[
            &first.cookie,
            &second.cookie,
            &first_token,
            &first.relay,
            &second.relay,
        ],
    );
    let foreign_row = login_row(&f, &first.relay);
    assert_eq!(foreign_row["failed"], true);
    assert!(foreign_row["result"].is_null());
    assert_eq!(login_row(&f, &second.relay)["failed"], false);
    assert!(login_row(&f, &second.relay)["result"].is_null());
    let still_foreign = f
        .core
        .saml_source_browser_return(&source.id, Some(&first.cookie), Some(&first_token))
        .unwrap_err();
    assert_eq!(still_foreign.status.as_u16(), 400);
    assert_ne!(still_foreign.code, "source_browser_mismatch");
    assert!(finish(&f, &first.credential, true).is_err());
    assert_eq!(failed_audits(&f), foreign_audits + 1);
    assert_eq!(accounts(&f), before);

    // Success: the POST omits the Lax start cookie. Both cookies confirm on the return.
    // Replay of that return or the ACS leaves the confirmed login in place.
    let started = browser_begin(&f, &source);
    let signed = relay_response(&upstream, &source, &acs, &started.request);
    let posted = post_acs(&f.core, &source.id, &started.relay, &signed).await;
    assert_eq!(posted.status, 303);
    assert_eq!(posted.referrer, "no-referrer");
    assert!(posted.location.ends_with("/saml/sources/enterprise/return"));
    let cookie_header = posted.set_cookie.as_deref().unwrap();
    assert!(cookie_header.contains("HttpOnly"));
    assert!(cookie_header.contains("SameSite=Lax"));
    assert!(!cookie_header.contains("SameSite=None"));
    assert!(!cookie_header.contains("Secure"));
    assert!(cookie_header.contains("Max-Age=600"));
    assert!(cookie_header.contains("Path=/"));
    let (return_name, token) = cookie_assignment(cookie_header);
    assert_eq!(return_name, "riauth_source_return");
    assert!(!posted.body.contains(&token));
    assert!(!posted.location.contains(&token));
    assert!(!posted.location.contains(&started.relay));
    assert!(!posted.body.contains(&started.cookie));
    let waiting = login_row(&f, &started.relay);
    assert_eq!(waiting["failed"], false);
    assert_eq!(waiting["browser_return_confirmed"], false);
    assert_eq!(waiting["result"]["subject"], "opaque-subject");
    assert_eq!(waiting["browser_return"], digest(&token));
    assert_ne!(waiting["browser_return"], token);
    assert_eq!(
        f.core.store.list::<String>("source_returns").unwrap().len(),
        1
    );
    let early = finish(&f, &started.credential, false).unwrap_err();
    assert_eq!(early.status.as_u16(), 401);
    assert_eq!(login_row(&f, &started.relay)["failed"], false);
    // The start cookie alone, or an oversized return cookie, must not end the login.
    let start_only = f
        .core
        .saml_source_browser_return(&source.id, Some(&started.cookie), None)
        .unwrap_err();
    assert_eq!(start_only.status.as_u16(), 400);
    let oversized = f
        .core
        .saml_source_browser_return(&source.id, Some(&started.cookie), Some(&"x".repeat(257)))
        .unwrap_err();
    assert_eq!(oversized.status.as_u16(), 400);
    hides(&oversized, &[&started.cookie, &token]);
    assert_eq!(login_row(&f, &started.relay)["failed"], false);
    assert_eq!(
        login_row(&f, &started.relay)["browser_return_confirmed"],
        false
    );
    let prefetch = get_return(&f.core, &source.id, None).await;
    assert_eq!(prefetch.status, 303);
    assert!(prefetch.set_cookie.is_none());
    assert!(prefetch.location.ends_with("/account/sources/continue"));
    assert_eq!(
        login_row(&f, &started.relay)["browser_return_confirmed"],
        false
    );
    assert_eq!(login_row(&f, &started.relay)["failed"], false);
    let both = format!(
        "riauth_source={}; riauth_source_return={token}",
        started.cookie
    );
    let wrong_provider = get_return(&f.core, "other", Some(&both)).await;
    assert_eq!(wrong_provider.status, 303);
    assert!(wrong_provider.set_cookie.is_none());
    assert_eq!(
        login_row(&f, &started.relay)["browser_return_confirmed"],
        false
    );
    assert_eq!(
        f.core.store.list::<String>("source_returns").unwrap().len(),
        1
    );
    let confirmed = get_return(&f.core, &source.id, Some(&both)).await;
    assert_eq!(confirmed.status, 303);
    assert_eq!(confirmed.referrer, "no-referrer");
    assert!(confirmed.location.ends_with("/account/sources/continue"));
    assert!(!confirmed.location.contains(&token));
    let clearing = confirmed.set_cookie.as_deref().unwrap();
    assert!(clearing.starts_with("riauth_source_return="));
    assert!(clearing.contains("Max-Age=0"));
    let open = login_row(&f, &started.relay);
    assert_eq!(open["browser_return_confirmed"], true);
    assert!(open["browser_return"].is_null());
    assert_eq!(open["failed"], false);
    assert_eq!(open["result"]["subject"], "opaque-subject");
    assert!(
        f.core
            .store
            .list::<String>("source_returns")
            .unwrap()
            .is_empty()
    );
    let replay_return = f
        .core
        .saml_source_browser_return(&source.id, Some(&started.cookie), Some(&token))
        .unwrap_err();
    assert_eq!(replay_return.status.as_u16(), 400);
    assert_eq!(replay_return.code, "invalid_request");
    assert!(replay_return.to_string().contains("already used"));
    hides(&replay_return, &[&started.cookie, &token, &started.relay]);
    assert_eq!(
        login_row(&f, &started.relay)["browser_return_confirmed"],
        true
    );
    assert_eq!(
        login_row(&f, &started.relay)["result"]["subject"],
        "opaque-subject"
    );
    let replay_acs = post_acs(&f.core, &source.id, &started.relay, &signed).await;
    assert_ne!(replay_acs.status, 303);
    assert!(replay_acs.set_cookie.is_none());
    assert_eq!(login_row(&f, &started.relay)["failed"], false);
    assert_eq!(
        login_row(&f, &started.relay)["browser_return_confirmed"],
        true
    );
    assert_eq!(
        login_row(&f, &started.relay)["result"]["subject"],
        "opaque-subject"
    );
    let review = f
        .core
        .portal_source_review(Some(&started.credential))
        .unwrap();
    assert_eq!(review["status"], "review");
    assert_eq!(review["subject"], "opaque-subject");
    assert_eq!(accounts(&f), before);
    let session = finish(&f, &started.credential, true).unwrap();
    assert_eq!(session["status"], "complete");
    assert!(f.core.me(&text(&session, "session_token")).is_ok());
    let (users, sessions, links) = accounts(&f);
    let (users_before, sessions_before, links_before) = before;
    assert_eq!(users, users_before + 1);
    assert_eq!(sessions, sessions_before + 1);
    assert_eq!(links, links_before + 1);
    assert!(finish(&f, &started.credential, true).is_err());
    assert!(f.core.me(&text(&session, "session_token")).is_ok());
}

fn put_saml(f: &Fixture, source: &Source) {
    f.core
        .source_put(
            &f.admin,
            SourceInput {
                source: source.clone(),
                client_secret: None,
            },
        )
        .unwrap();
}

fn post(f: &Fixture, source: &Source, relay: &str, response: &str) -> riauth::error::Result<Value> {
    f.core.saml_source_callback(
        &source.id,
        vec![
            ("SAMLResponse".into(), STANDARD.encode(response)),
            ("RelayState".into(), relay.into()),
        ],
    )
}

fn saml_links(f: &Fixture) -> Vec<Value> {
    f.core
        .store
        .list::<Value>("source_links")
        .unwrap()
        .into_iter()
        .map(|(_, link)| link)
        .collect()
}

fn replay_entries(f: &Fixture) -> usize {
    f.core
        .store
        .list::<u64>("saml_source_replays")
        .unwrap()
        .len()
}

fn replacement_idp(name: &str) -> Upstream {
    let key = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();
    Upstream {
        cert: saml_tests::cert(&key, name),
        key,
        xmlsec: None,
        dir: tempfile::tempdir().unwrap(),
    }
}

/// Old and new pinned IdP certificates both verify. A removed certificate cannot
/// authenticate, and restoring it does not finish a login presented while that
/// certificate was absent. An unpresented login can still complete after the
/// original certificates are restored. Issuer, audience and link rules stay put.
#[test]
fn saml_source_certificate_rollover_checks_old_and_new_keys_stale_assertions_and_rollback_replay() {
    let f = Fixture::new();
    let (mut source, old) = install_saml(&f);
    let new_idp = replacement_idp("Replacement IdP");
    let acs = f.core.saml_source_callback_url(&source.id);
    source.saml.as_mut().unwrap().idp_certificates_pem =
        vec![old.cert.clone(), new_idp.cert.clone()];
    put_saml(&f, &source);

    let (relay, request, credential) = begin(&f, &source, None);
    let response = old.response(&source, &acs, &request, None, false);
    let body = post(&f, &source, &relay, &response).unwrap();
    assert_eq!(body["completed"], true);
    assert!(!body.to_string().contains("samlp:Response"));
    let first = finish(&f, &credential, true).unwrap();
    let user_id = text(&first["user"], "id");
    let session_old = text(&first, "session_token");
    assert!(f.core.me(&session_old).is_ok());

    let (relay, request, credential) = begin(&f, &source, None);
    let response = new_idp.response(&source, &acs, &request, None, false);
    assert_eq!(
        post(&f, &source, &relay, &response).unwrap()["completed"],
        true
    );
    let second = finish(&f, &credential, true).unwrap();
    assert_eq!(text(&second["user"], "id"), user_id);
    let session_new = text(&second, "session_token");
    assert!(f.core.me(&session_old).is_ok());
    assert!(f.core.me(&session_new).is_ok());
    let links = saml_links(&f);
    assert_eq!(links.len(), 1);
    assert_eq!(links[0]["source"], "enterprise");
    assert_eq!(links[0]["issuer"], source.issuer);
    assert_eq!(links[0]["subject"], "opaque-subject");
    assert_eq!(links[0]["user_id"], user_id);

    let (relay, request, credential) = begin(&f, &source, None);
    let response = new_idp.response(
        &source,
        &acs,
        &request,
        Some((
            "urn:example:riauth-sp</saml:Audience>",
            "urn:attacker</saml:Audience>",
        )),
        false,
    );
    let audience = post(&f, &source, &relay, &response).unwrap();
    assert_eq!(audience["completed"], false);
    assert!(!audience.to_string().contains("samlp:Response"));
    assert!(finish(&f, &credential, true).is_err());
    assert_eq!(saml_links(&f).len(), 1);
    assert!(f.core.me(&session_old).is_ok());

    let mut foreign = source.clone();
    foreign.issuer = "urn:example:other-idp".into();
    let rejected = f
        .core
        .source_put(
            &f.admin,
            SourceInput {
                source: foreign,
                client_secret: None,
            },
        )
        .unwrap_err();
    assert_eq!(rejected.code, "conflict");
    hides(
        &rejected,
        &[&old.cert, &new_idp.cert, "urn:example:other-idp"],
    );
    assert_eq!(saml_links(&f)[0]["issuer"], source.issuer);
    assert!(f.core.me(&session_old).is_ok());
    assert!(f.core.me(&f.admin).is_ok());

    let (parked, parked_request, parked_credential) = begin(&f, &source, None);
    let parked_response = old.response(&source, &acs, &parked_request, None, false);
    source.saml.as_mut().unwrap().idp_certificates_pem = vec![new_idp.cert.clone()];
    put_saml(&f, &source);
    assert_eq!(login_row(&f, &parked)["claimed"], false);
    assert_eq!(login_row(&f, &parked)["failed"], false);
    assert!(f.core.me(&session_old).is_err());
    assert!(f.core.me(&session_new).is_err());
    assert!(f.core.me(&f.admin).is_ok());
    source.saml.as_mut().unwrap().idp_certificates_pem =
        vec![old.cert.clone(), new_idp.cert.clone()];
    put_saml(&f, &source);
    assert_eq!(
        post(&f, &source, &parked, &parked_response).unwrap()["completed"],
        true
    );
    let parked_session = finish(&f, &parked_credential, true).unwrap();
    assert_eq!(text(&parked_session["user"], "id"), user_id);
    let session_parked = text(&parked_session, "session_token");
    assert!(f.core.me(&session_parked).is_ok());

    source.saml.as_mut().unwrap().idp_certificates_pem = vec![new_idp.cert.clone()];
    put_saml(&f, &source);
    assert!(f.core.me(&session_parked).is_err());
    assert_eq!(saml_links(&f)[0]["subject"], "opaque-subject");
    assert_eq!(saml_links(&f)[0]["issuer"], source.issuer);
    let fresh = {
        let (relay, request, credential) = begin(&f, &source, None);
        let response = new_idp.response(&source, &acs, &request, None, false);
        assert_eq!(
            post(&f, &source, &relay, &response).unwrap()["completed"],
            true
        );
        finish(&f, &credential, true).unwrap()
    };
    assert_eq!(text(&fresh["user"], "id"), user_id);
    let session_cutover = text(&fresh, "session_token");
    assert!(f.core.me(&session_cutover).is_ok());

    let replays_before_stale = replay_entries(&f);
    let (stale, stale_request, stale_credential) = begin(&f, &source, None);
    let stale_response = old.response(&source, &acs, &stale_request, None, false);
    let stale_body = post(&f, &source, &stale, &stale_response).unwrap();
    assert_eq!(stale_body["completed"], false);
    assert!(!stale_body.to_string().contains(&old.cert));
    let stale_row = login_row(&f, &stale);
    assert_eq!(stale_row["failed"], true);
    assert_eq!(stale_row["claimed"], true);
    assert!(stale_row["result"].is_null());
    assert!(finish(&f, &stale_credential, true).is_err());
    assert!(post(&f, &source, &stale, &stale_response).is_err());
    assert_eq!(replay_entries(&f), replays_before_stale);
    assert!(f.core.me(&session_cutover).is_ok());

    source.saml.as_mut().unwrap().idp_certificates_pem = vec![old.cert.clone()];
    put_saml(&f, &source);
    assert!(f.core.me(&session_cutover).is_err());
    assert!(f.core.me(&f.admin).is_ok());
    assert!(finish(&f, &stale_credential, true).is_err());
    let stale_replay = post(&f, &source, &stale, &stale_response).unwrap_err();
    assert_eq!(stale_replay.code, "invalid_request");
    assert!(stale_replay.to_string().contains("already used"));
    hides(&stale_replay, &[&old.cert, &stale, &stale_response]);
    assert_eq!(login_row(&f, &stale)["failed"], true);
    assert_eq!(saml_links(&f).len(), 1);
    let restored = {
        let (relay, request, credential) = begin(&f, &source, None);
        let response = old.response(&source, &acs, &request, None, false);
        assert_eq!(
            post(&f, &source, &relay, &response).unwrap()["completed"],
            true
        );
        finish(&f, &credential, true).unwrap()
    };
    assert_eq!(text(&restored["user"], "id"), user_id);
    let session_restored = text(&restored, "session_token");
    assert!(f.core.me(&session_restored).is_ok());

    let (retired, retired_request, retired_credential) = begin(&f, &source, None);
    let retired_response = old.response(&source, &acs, &retired_request, None, false);
    source.saml.as_mut().unwrap().idp_certificates_pem = vec![new_idp.cert.clone()];
    put_saml(&f, &source);
    assert!(f.core.me(&session_restored).is_err());
    let replays_before_retire = replay_entries(&f);
    let retired_error = post(&f, &source, &retired, &retired_response).unwrap_err();
    assert_eq!(retired_error.code, "invalid_request");
    assert!(retired_error.to_string().contains("SAML source changed"));
    hides(
        &retired_error,
        &[&old.cert, &new_idp.cert, &retired, &retired_response],
    );
    let retired_row = login_row(&f, &retired);
    assert_eq!(retired_row["failed"], true);
    assert_eq!(retired_row["claimed"], true);
    assert!(retired_row["result"].is_null());
    assert_eq!(replay_entries(&f), replays_before_retire);
    let retired_again = post(&f, &source, &retired, &retired_response).unwrap_err();
    assert_eq!(retired_again.code, "invalid_request");
    assert!(retired_again.to_string().contains("already used"));
    source.saml.as_mut().unwrap().idp_certificates_pem = vec![old.cert.clone()];
    put_saml(&f, &source);
    assert!(finish(&f, &retired_credential, true).is_err());
    let retired_rollback = post(&f, &source, &retired, &retired_response).unwrap_err();
    assert_eq!(retired_rollback.code, "invalid_request");
    assert!(retired_rollback.to_string().contains("already used"));
    hides(&retired_rollback, &[&old.cert, &retired, &retired_response]);
    assert_eq!(login_row(&f, &retired)["failed"], true);
    assert_eq!(saml_links(&f).len(), 1);
    let (relay, request, credential) = begin(&f, &source, None);
    let response = old.response(&source, &acs, &request, None, false);
    assert_eq!(
        post(&f, &source, &relay, &response).unwrap()["completed"],
        true
    );
    assert_eq!(
        text(&finish(&f, &credential, true).unwrap()["user"], "id"),
        user_id
    );
}

#[test]
fn saml_browser_return_claim_keeps_provider_burn_replay_and_retirement() {
    let f = Fixture::new();
    let (mut source, upstream) = install_saml(&f);
    let acs = f.core.saml_source_callback_url(&source.id);
    let issue = |source: &Source| {
        let started = browser_begin(&f, source);
        let response = relay_response(&upstream, source, &acs, &started.request);
        let token = text(
            &post(&f, source, &started.relay, &response).unwrap(),
            "browser_return",
        );
        (started, token)
    };

    let (first, first_token) = issue(&source);
    let before = f.snapshot().unwrap();
    let wrong_provider = f
        .core
        .saml_source_browser_return("other", Some(&first.cookie), Some(&first_token))
        .unwrap_err();
    assert_eq!(wrong_provider.code, "invalid_request");
    f.assert_snapshot(&before);
    assert_eq!(
        f.core
            .store
            .get::<String>("source_returns", &digest(&first_token))
            .unwrap(),
        Some(digest(&first.relay))
    );

    let audits = failed_audits(&f);
    let mismatch = f
        .core
        .saml_source_browser_return(&source.id, Some("wrong-browser"), Some(&first_token))
        .unwrap_err();
    assert_eq!(mismatch.code, "source_browser_mismatch");
    hides(&mismatch, &[&first.cookie, &first_token]);
    let row = login_row(&f, &first.relay);
    assert_eq!(row["failed"], true);
    assert!(row["result"].is_null());
    assert!(row["browser_return"].is_null());
    assert!(
        f.core
            .store
            .get::<String>("source_returns", &digest(&first_token))
            .unwrap()
            .is_none()
    );
    assert_eq!(failed_audits(&f), audits + 1);
    let replay = f
        .core
        .saml_source_browser_return(&source.id, Some(&first.cookie), Some(&first_token))
        .unwrap_err();
    assert_eq!(replay.code, "invalid_request");
    assert_eq!(failed_audits(&f), audits + 1);

    let (retiring, retired_token) = issue(&source);
    source.enabled = false;
    put_saml(&f, &source);
    let audits = failed_audits(&f);
    let retired = f
        .core
        .saml_source_browser_return(&source.id, Some(&retiring.cookie), Some(&retired_token))
        .unwrap_err();
    assert_eq!(retired.message, "SAML source changed; restart login");
    hides(&retired, &[&retiring.cookie, &retired_token]);
    let row = login_row(&f, &retiring.relay);
    assert_eq!(row["failed"], true);
    assert!(row["result"].is_null());
    assert!(row["browser_return"].is_null());
    assert!(
        f.core
            .store
            .get::<String>("source_returns", &digest(&retired_token))
            .unwrap()
            .is_none()
    );
    assert_eq!(failed_audits(&f), audits + 1);
    source.enabled = true;
    put_saml(&f, &source);
    assert_eq!(
        f.core
            .saml_source_browser_return(&source.id, Some(&retiring.cookie), Some(&retired_token))
            .unwrap_err()
            .code,
        "invalid_request"
    );
    assert_eq!(failed_audits(&f), audits + 1);

    let (confirmed, confirmed_token) = issue(&source);
    f.core
        .saml_source_browser_return(&source.id, Some(&confirmed.cookie), Some(&confirmed_token))
        .unwrap();
    let row = login_row(&f, &confirmed.relay);
    assert_eq!(row["browser_return_confirmed"], true);
    assert_eq!(row["failed"], false);
    assert!(row["browser_return"].is_null());
    assert_eq!(
        f.core
            .saml_source_browser_return(&source.id, Some(&confirmed.cookie), Some(&confirmed_token))
            .unwrap_err()
            .code,
        "invalid_request"
    );
    assert_eq!(failed_audits(&f), audits + 1);

    let (expired, expired_token) = issue(&source);
    let mut expired_row = login_row(&f, &expired.relay);
    expired_row["expires_at"] = json!(now() - 1);
    f.core
        .store
        .write(|tx| tx.put("source_logins", &digest(&expired.relay), &expired_row))
        .unwrap();
    assert_eq!(
        f.core
            .saml_source_browser_return(&source.id, Some(&expired.cookie), Some(&expired_token))
            .unwrap_err()
            .code,
        "invalid_request"
    );
    assert!(
        f.core
            .store
            .get::<String>("source_returns", &digest(&expired_token))
            .unwrap()
            .is_none()
    );
    assert_eq!(failed_audits(&f), audits + 1);
}

/// A browser return presented while the pinned source differs ends that login.
/// Restoring the previous certificate does not confirm it. A return that was
/// never presented during the change can still confirm after the original
/// certificate is restored. Issuer and subject stay on the first account.
#[test]
fn saml_browser_return_ends_when_pinned_source_changes_and_restore_cannot_confirm() {
    let f = Fixture::new();
    let (mut source, old) = install_saml(&f);
    let replacement = replacement_idp("Replacement IdP");
    let acs = f.core.saml_source_callback_url(&source.id);

    // Unpresented return: a certificate round trip leaves it confirmable.
    let parked = browser_begin(&f, &source);
    let parked_xml = old.response(&source, &acs, &parked.request, None, false);
    let parked_body = post(&f, &source, &parked.relay, &parked_xml).unwrap();
    assert_eq!(parked_body["completed"], true);
    let parked_token = text(&parked_body, "browser_return");
    assert!(!parked_body.to_string().contains(&old.cert));
    assert_eq!(
        login_row(&f, &parked.relay)["browser_return"],
        digest(&parked_token)
    );
    source.saml.as_mut().unwrap().idp_certificates_pem = vec![replacement.cert.clone()];
    put_saml(&f, &source);
    assert_eq!(login_row(&f, &parked.relay)["failed"], false);
    assert_eq!(
        login_row(&f, &parked.relay)["browser_return_confirmed"],
        false
    );
    assert_eq!(
        f.core.store.list::<String>("source_returns").unwrap().len(),
        1
    );
    source.saml.as_mut().unwrap().idp_certificates_pem = vec![old.cert.clone()];
    put_saml(&f, &source);
    f.core
        .saml_source_browser_return(&source.id, Some(&parked.cookie), Some(&parked_token))
        .unwrap();
    let parked_row = login_row(&f, &parked.relay);
    assert_eq!(parked_row["browser_return_confirmed"], true);
    assert_eq!(parked_row["failed"], false);
    assert_eq!(parked_row["result"]["subject"], "opaque-subject");
    let first = finish(&f, &parked.credential, true).unwrap();
    let user_id = text(&first["user"], "id");
    let session = text(&first, "session_token");
    assert!(f.core.me(&session).is_ok());
    assert_eq!(saml_links(&f).len(), 1);
    assert_eq!(saml_links(&f)[0]["issuer"], source.issuer);
    assert_eq!(saml_links(&f)[0]["subject"], "opaque-subject");
    assert_eq!(saml_links(&f)[0]["user_id"], user_id);
    assert!(
        f.core
            .store
            .list::<String>("source_returns")
            .unwrap()
            .is_empty()
    );

    // Presented during the change: missing and unknown tokens do not end it.
    let started = browser_begin(&f, &source);
    let xml = old.response(&source, &acs, &started.request, None, false);
    let body = post(&f, &source, &started.relay, &xml).unwrap();
    assert_eq!(body["completed"], true);
    let token = text(&body, "browser_return");
    assert_eq!(login_row(&f, &started.relay)["claimed"], true);
    assert_eq!(login_row(&f, &started.relay)["failed"], false);
    assert_eq!(
        login_row(&f, &started.relay)["result"]["subject"],
        "opaque-subject"
    );
    assert_eq!(
        login_row(&f, &started.relay)["browser_return"],
        digest(&token)
    );
    source.saml.as_mut().unwrap().idp_certificates_pem = vec![replacement.cert.clone()];
    put_saml(&f, &source);
    assert!(f.core.me(&session).is_err());
    assert!(f.core.me(&f.admin).is_ok());
    assert_eq!(login_row(&f, &started.relay)["failed"], false);
    let missing = f
        .core
        .saml_source_browser_return(&source.id, Some(&started.cookie), None)
        .unwrap_err();
    assert_eq!(missing.code, "invalid_request");
    assert!(missing.to_string().contains("already used"));
    hides(
        &missing,
        &[&started.cookie, &token, &old.cert, &replacement.cert],
    );
    let unknown = f
        .core
        .saml_source_browser_return(&source.id, Some(&started.cookie), Some("not-a-return"))
        .unwrap_err();
    assert_eq!(unknown.code, "invalid_request");
    hides(&unknown, &[&started.cookie, &token, &started.relay]);
    assert_eq!(login_row(&f, &started.relay)["failed"], false);
    assert_eq!(
        login_row(&f, &started.relay)["result"]["subject"],
        "opaque-subject"
    );
    assert_eq!(
        f.core.store.list::<String>("source_returns").unwrap().len(),
        1
    );
    let settled = accounts(&f);
    let audits = failed_audits(&f);
    let replays = replay_entries(&f);
    let retired = f
        .core
        .saml_source_browser_return(&source.id, Some(&started.cookie), Some(&token))
        .unwrap_err();
    assert_eq!(retired.status.as_u16(), 400);
    assert_eq!(retired.code, "invalid_request");
    assert!(retired.to_string().contains("SAML source changed"));
    hides(
        &retired,
        &[
            &old.cert,
            &replacement.cert,
            &started.cookie,
            &token,
            &started.relay,
            &xml,
        ],
    );
    let row = login_row(&f, &started.relay);
    assert_eq!(row["failed"], true);
    assert_eq!(row["claimed"], true);
    assert!(row["result"].is_null());
    assert!(row["browser_return"].is_null());
    assert_eq!(row["browser_return_confirmed"], false);
    assert!(
        f.core
            .store
            .list::<String>("source_returns")
            .unwrap()
            .is_empty()
    );
    assert_eq!(failed_audits(&f), audits + 1);
    assert_eq!(replay_entries(&f), replays);
    assert_eq!(accounts(&f), settled);
    let replay = f
        .core
        .saml_source_browser_return(&source.id, Some(&started.cookie), Some(&token))
        .unwrap_err();
    assert_eq!(replay.code, "invalid_request");
    assert!(replay.to_string().contains("already used"));
    hides(
        &replay,
        &[&started.cookie, &token, &started.relay, &old.cert, &xml],
    );
    assert_eq!(login_row(&f, &started.relay)["failed"], true);

    source.saml.as_mut().unwrap().idp_certificates_pem = vec![old.cert.clone()];
    put_saml(&f, &source);
    assert!(f.core.me(&f.admin).is_ok());
    let restored = f
        .core
        .saml_source_browser_return(&source.id, Some(&started.cookie), Some(&token))
        .unwrap_err();
    assert_eq!(restored.code, "invalid_request");
    assert!(restored.to_string().contains("already used"));
    hides(
        &restored,
        &[
            &old.cert,
            &replacement.cert,
            &token,
            &started.cookie,
            &started.relay,
            &xml,
        ],
    );
    assert!(finish(&f, &started.credential, true).is_err());
    assert_eq!(
        f.core
            .portal_source_review(Some(&started.credential))
            .unwrap_err()
            .code,
        "source_login_expired"
    );
    assert_eq!(login_row(&f, &started.relay)["failed"], true);
    assert!(login_row(&f, &started.relay)["result"].is_null());
    assert_eq!(saml_links(&f).len(), 1);
    assert_eq!(saml_links(&f)[0]["issuer"], "urn:example:enterprise-idp");
    assert_eq!(saml_links(&f)[0]["subject"], "opaque-subject");
    assert_eq!(saml_links(&f)[0]["user_id"], user_id);
    assert_eq!(accounts(&f), settled);

    let (relay, request, credential) = begin(&f, &source, None);
    let response = old.response(&source, &acs, &request, None, false);
    assert_eq!(
        post(&f, &source, &relay, &response).unwrap()["completed"],
        true
    );
    let again = finish(&f, &credential, true).unwrap();
    assert_eq!(text(&again["user"], "id"), user_id);
    assert!(f.core.me(&text(&again, "session_token")).is_ok());
    assert_eq!(saml_links(&f).len(), 1);
    assert_eq!(saml_links(&f)[0]["subject"], "opaque-subject");
    assert_eq!(saml_links(&f)[0]["issuer"], source.issuer);
    assert_eq!(saml_links(&f)[0]["user_id"], user_id);
}

/// Disabling a source changes its fingerprint. A SAML response or browser
/// return presented while it is disabled ends that login, so enabling the
/// source again cannot continue it. A login that was not presented can still
/// complete after re-enable, and the issuer and subject stay on one link.
#[test]
fn saml_login_presented_while_source_disabled_cannot_continue_after_reenable() {
    let f = Fixture::new();
    let (mut source, old) = install_saml(&f);
    let acs = f.core.saml_source_callback_url(&source.id);

    let (parked, parked_request, parked_credential) = begin(&f, &source, None);
    let parked_response = old.response(&source, &acs, &parked_request, None, false);
    source.enabled = false;
    put_saml(&f, &source);
    assert!(
        f.core
            .source_start(
                &source.id,
                Start {
                    link: false,
                    authentication_transaction: None,
                },
                None,
            )
            .is_err()
    );
    assert_eq!(login_row(&f, &parked)["failed"], false);
    assert_eq!(login_row(&f, &parked)["claimed"], false);
    source.enabled = true;
    put_saml(&f, &source);
    assert_eq!(
        post(&f, &source, &parked, &parked_response).unwrap()["completed"],
        true
    );
    let first = finish(&f, &parked_credential, true).unwrap();
    let user_id = text(&first["user"], "id");
    let session = text(&first, "session_token");
    assert!(f.core.me(&session).is_ok());
    assert_eq!(saml_links(&f).len(), 1);
    assert_eq!(saml_links(&f)[0]["issuer"], source.issuer);
    assert_eq!(saml_links(&f)[0]["subject"], "opaque-subject");
    assert_eq!(saml_links(&f)[0]["user_id"], user_id);

    let (retired, retired_request, retired_credential) = begin(&f, &source, None);
    let retired_response = old.response(&source, &acs, &retired_request, None, false);
    source.enabled = false;
    put_saml(&f, &source);
    assert!(f.core.me(&session).is_err());
    assert!(f.core.me(&f.admin).is_ok());
    let replays = replay_entries(&f);
    let retired_error = post(&f, &source, &retired, &retired_response).unwrap_err();
    assert_eq!(retired_error.code, "invalid_request");
    assert!(retired_error.to_string().contains("SAML source changed"));
    hides(&retired_error, &[&old.cert, &retired, &retired_response]);
    let retired_row = login_row(&f, &retired);
    assert_eq!(retired_row["failed"], true);
    assert_eq!(retired_row["claimed"], true);
    assert!(retired_row["result"].is_null());
    assert_eq!(replay_entries(&f), replays);
    let retired_again = post(&f, &source, &retired, &retired_response).unwrap_err();
    assert_eq!(retired_again.code, "invalid_request");
    assert!(retired_again.to_string().contains("already used"));
    source.enabled = true;
    put_saml(&f, &source);
    assert!(finish(&f, &retired_credential, true).is_err());
    let retired_replay = post(&f, &source, &retired, &retired_response).unwrap_err();
    assert_eq!(retired_replay.code, "invalid_request");
    assert!(retired_replay.to_string().contains("already used"));
    hides(&retired_replay, &[&old.cert, &retired, &retired_response]);
    assert_eq!(login_row(&f, &retired)["failed"], true);
    assert_eq!(saml_links(&f).len(), 1);
    assert_eq!(saml_links(&f)[0]["subject"], "opaque-subject");
    assert_eq!(saml_links(&f)[0]["user_id"], user_id);

    let waiting = browser_begin(&f, &source);
    let waiting_body = post(
        &f,
        &source,
        &waiting.relay,
        &old.response(&source, &acs, &waiting.request, None, false),
    )
    .unwrap();
    assert_eq!(waiting_body["completed"], true);
    let waiting_token = text(&waiting_body, "browser_return");
    source.enabled = false;
    put_saml(&f, &source);
    let untouched = f
        .core
        .saml_source_browser_return(&source.id, Some(&waiting.cookie), None)
        .unwrap_err();
    assert_eq!(untouched.code, "invalid_request");
    assert!(untouched.to_string().contains("already used"));
    hides(&untouched, &[&waiting.cookie, &waiting_token, &old.cert]);
    assert_eq!(login_row(&f, &waiting.relay)["failed"], false);
    assert_eq!(
        login_row(&f, &waiting.relay)["result"]["subject"],
        "opaque-subject"
    );
    assert_eq!(
        f.core.store.list::<String>("source_returns").unwrap().len(),
        1
    );
    let returned = f
        .core
        .saml_source_browser_return(&source.id, Some(&waiting.cookie), Some(&waiting_token))
        .unwrap_err();
    assert_eq!(returned.code, "invalid_request");
    assert!(returned.to_string().contains("SAML source changed"));
    hides(
        &returned,
        &[&waiting.cookie, &waiting_token, &waiting.relay, &old.cert],
    );
    let returned_row = login_row(&f, &waiting.relay);
    assert_eq!(returned_row["failed"], true);
    assert!(returned_row["result"].is_null());
    assert!(returned_row["browser_return"].is_null());
    assert_eq!(returned_row["browser_return_confirmed"], false);
    assert!(
        f.core
            .store
            .list::<String>("source_returns")
            .unwrap()
            .is_empty()
    );
    source.enabled = true;
    put_saml(&f, &source);
    let returned_again = f
        .core
        .saml_source_browser_return(&source.id, Some(&waiting.cookie), Some(&waiting_token))
        .unwrap_err();
    assert_eq!(returned_again.code, "invalid_request");
    assert!(returned_again.to_string().contains("already used"));
    hides(
        &returned_again,
        &[&waiting.cookie, &waiting_token, &old.cert],
    );
    assert!(finish(&f, &waiting.credential, true).is_err());
    assert_eq!(
        f.core
            .portal_source_review(Some(&waiting.credential))
            .unwrap_err()
            .code,
        "source_login_expired"
    );
    assert_eq!(saml_links(&f).len(), 1);

    let parked_return = browser_begin(&f, &source);
    let parked_return_body = post(
        &f,
        &source,
        &parked_return.relay,
        &old.response(&source, &acs, &parked_return.request, None, false),
    )
    .unwrap();
    let parked_return_token = text(&parked_return_body, "browser_return");
    source.enabled = false;
    put_saml(&f, &source);
    assert_eq!(login_row(&f, &parked_return.relay)["failed"], false);
    source.enabled = true;
    put_saml(&f, &source);
    f.core
        .saml_source_browser_return(
            &source.id,
            Some(&parked_return.cookie),
            Some(&parked_return_token),
        )
        .unwrap();
    let resumed = finish(&f, &parked_return.credential, true).unwrap();
    assert_eq!(text(&resumed["user"], "id"), user_id);
    assert!(f.core.me(&text(&resumed, "session_token")).is_ok());
    assert_eq!(saml_links(&f).len(), 1);
    assert_eq!(saml_links(&f)[0]["issuer"], "urn:example:enterprise-idp");
    assert_eq!(saml_links(&f)[0]["subject"], "opaque-subject");
    assert_eq!(saml_links(&f)[0]["user_id"], user_id);
}
