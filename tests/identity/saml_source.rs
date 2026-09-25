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
    let url = url::Url::parse(start["authorization_url"].as_str().unwrap()).unwrap();
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
    assert_eq!(
        root.attribute("AssertionConsumerServiceURL"),
        Some(f.core.saml_source_callback_url(&source.id).as_str())
    );
    (
        fields["RelayState"].clone(),
        root.attribute("ID").unwrap().into(),
        text(&start["credential"], "token"),
    )
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
