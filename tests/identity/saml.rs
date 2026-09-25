use super::*;
use base64::engine::general_purpose::STANDARD;
use openssl::{
    asn1::Asn1Time,
    hash::MessageDigest,
    pkey::{PKey, Private},
    rsa::Rsa,
    sign::{Signer, Verifier},
    x509::{X509, X509NameBuilder},
};
use riauth::{
    browser::{BrowserDecision, BrowserReply},
    saml::{Attribute, NameIdFormat, Reply, Settings},
    signin,
};
use std::{
    io::{Read, Write},
    path::Path,
};
const P: &str = "urn:oasis:names:tc:SAML:2.0:protocol";
const A: &str = "urn:oasis:names:tc:SAML:2.0:assertion";
const ALG: &str = "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256";
const SP: &str = "https://sp.example.test/metadata";
const ACS: &str = "https://sp.example.test/acs";
const ENDPOINT: &str = "http://localhost:9000/saml/saml-app/sso";
pub(super) fn cert(key: &PKey<Private>, name: &str) -> String {
    let mut subject = X509NameBuilder::new().unwrap();
    subject.append_entry_by_text("CN", name).unwrap();
    let subject = subject.build();
    let mut c = X509::builder().unwrap();
    c.set_version(2).unwrap();
    c.set_serial_number(
        &openssl::bn::BigNum::from_u32(1)
            .unwrap()
            .to_asn1_integer()
            .unwrap(),
    )
    .unwrap();
    c.set_subject_name(&subject).unwrap();
    c.set_issuer_name(&subject).unwrap();
    c.set_pubkey(key).unwrap();
    c.set_not_before(&Asn1Time::days_from_now(0).unwrap())
        .unwrap();
    c.set_not_after(&Asn1Time::days_from_now(1).unwrap())
        .unwrap();
    c.sign(key, MessageDigest::sha256()).unwrap();
    String::from_utf8(c.build().to_pem().unwrap()).unwrap()
}
fn timestamp(at: u64) -> String {
    time::OffsetDateTime::from_unix_timestamp(at as i64)
        .unwrap()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap()
}
fn authn(id: &str, extra: &str) -> String {
    format!(
        "<samlp:AuthnRequest xmlns:samlp=\"{P}\" xmlns:saml=\"{A}\" ID=\"{id}\" Version=\"2.0\" IssueInstant=\"{}\" Destination=\"{ENDPOINT}\" AssertionConsumerServiceURL=\"{ACS}\" ProtocolBinding=\"urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST\" {extra}><saml:Issuer>{SP}</saml:Issuer><samlp:NameIDPolicy Format=\"{}\" AllowCreate=\"true\"/></samlp:AuthnRequest>",
        timestamp(now()),
        NameIdFormat::Persistent.uri()
    )
}
fn redirect(xml: &str, key: &PKey<Private>, field: &str) -> String {
    let mut c = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
    c.write_all(xml.as_bytes()).unwrap();
    let encoded = STANDARD.encode(c.finish().unwrap());
    let mut query = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs([
            (field, encoded.as_str()),
            ("RelayState", "state & exact +"),
            ("SigAlg", ALG),
        ])
        .finish();
    let mut signer = Signer::new(MessageDigest::sha256(), key).unwrap();
    signer.update(query.as_bytes()).unwrap();
    let signature = STANDARD.encode(signer.sign_to_vec().unwrap());
    query.push('&');
    query.push_str(
        &url::form_urlencoded::Serializer::new(String::new())
            .append_pair("Signature", &signature)
            .finish(),
    );
    query
}
fn waiting(reply: Reply) -> (String, String, String) {
    let Reply::Waiting(reply) = reply else {
        panic!("expected terminal handoff")
    };
    let code = text(&reply.body, "user_code");
    let id = reply
        .refresh
        .unwrap()
        .rsplit('/')
        .next()
        .unwrap()
        .to_owned();
    let cookie = reply
        .cookies
        .iter()
        .find(|s| s.starts_with("riauth_saml="))
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1
        .to_owned();
    (code, id, cookie)
}
fn body(reply: Reply) -> (String, Vec<String>) {
    let Reply::Post {
        target,
        fields,
        cookies,
    } = reply
    else {
        panic!("expected SAML POST response")
    };
    assert_eq!(target, ACS);
    let mut xml = None;
    for (k, v) in fields {
        if k == "SAMLResponse" {
            xml = Some(String::from_utf8(STANDARD.decode(v).unwrap()).unwrap());
        } else {
            assert_eq!(k, "RelayState");
            assert_eq!(v, "state & exact +");
        }
    }
    (xml.unwrap(), cookies)
}
fn approve(f: &Fixture, session: &str, code: &str, transaction: Option<String>, remember: bool) {
    f.core
        .browser_decide(
            session,
            BrowserDecision {
                code: code.into(),
                approve: true,
                transaction_id: transaction,
                remember,
            },
        )
        .unwrap();
}
pub(super) fn xmlsec(binary: &Path, args: &[&str]) {
    // XMLSec 1.3 uses strict key selection by default; older releases have
    // no flag and already use the single explicitly loaded fixture key.
    let help = std::process::Command::new(binary)
        .arg("--help-all")
        .output()
        .unwrap();
    let has_lax = String::from_utf8_lossy(&help.stdout).contains("--lax-key-search");
    let args = args
        .iter()
        .filter(|arg| has_lax || **arg != "--lax-key-search");
    let result = std::process::Command::new(binary)
        .args(args)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "xmlsec failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}
fn verify_xml(binary: &Path, xml: &str, cert: &Path, dir: &Path, assertion: bool) {
    let file = dir.join("verify.xml");
    std::fs::write(&file, xml).unwrap();
    xmlsec(
        binary,
        &[
            "--verify",
            "--pubkey-cert-pem",
            cert.to_str().unwrap(),
            "--trusted-pem",
            cert.to_str().unwrap(),
            "--enabled-reference-uris",
            "same-doc",
            "--id-attr:ID",
            "Response",
            "--id-attr:ID",
            "Assertion",
            "--id-attr:ID",
            "EntityDescriptor",
            "--node-xpath",
            if assertion {
                "//*[local-name()='Assertion']/*[local-name()='Signature']"
            } else {
                "/*/*[local-name()='Signature']"
            },
            file.to_str().unwrap(),
        ],
    );
}
fn exercise(independent: Option<&Path>) {
    let f = Fixture::new();
    let session = f.user("saml-user");
    let other = f.user("saml-other");
    f.core.create_group(&f.admin, "saml-group").unwrap();
    f.core
        .group_member(&f.admin, "saml-group", "saml-user", true)
        .unwrap();
    f.core
        .update_user(
            &f.admin,
            "saml-user",
            UserPatch {
                display_name: Some("Name <&\" escape".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let idp_keys: crypto::Keys = f.core.store.get("meta", "keys").unwrap().unwrap();
    let idp_key = PKey::private_key_from_pem(idp_keys.active.pem.as_bytes()).unwrap();
    let idp_cert = cert(&idp_key, "riAuth fixture");
    let sp_key = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();
    let sp_cert = cert(&sp_key, "SP fixture");
    let mut settings = Settings {
        sp_entity_id: SP.into(),
        acs_urls: vec![ACS.into()],
        acs_indices: [(7, ACS.into())].into(),
        idp_entity_id: None,
        idp_certificate_pem: idp_cert.clone(),
        sp_certificates_pem: vec![sp_cert.clone()],
        encryption_certificate_pem: None,
        name_id_format: NameIdFormat::Persistent,
        attributes: vec![
            Attribute {
                name: "urn:test:name".into(),
                claim: "name".into(),
                friendly_name: Some("Display name".into()),
                required: true,
            },
            Attribute {
                name: "urn:test:groups".into(),
                claim: "groups".into(),
                friendly_name: None,
                required: true,
            },
        ],
        slo_redirect_url: Some("https://sp.example.test/logout".into()),
        slo_post_url: Some("https://sp.example.test/logout".into()),
        idp_initiated: false,
        default_relay_state: None,
        assertion_ttl: 120,
    };
    f.core
        .create_client(
            &f.admin,
            NewClient {
                client_id: "saml-app".into(),
                name: "SAML fixture".into(),
                confidential: false,
                redirect_uris: vec![],
                scopes: strings(&["openid", "saml", "profile", "groups"]),
                allowed_groups: strings(&["saml-group"]),
                require_mfa: false,
                service: false,
                settings: ProviderSettings {
                    saml: Some(settings.clone()),
                    ..Default::default()
                },
            },
        )
        .unwrap();
    let cert_file = f._dir.path().join("idp.pem");
    std::fs::write(&cert_file, &idp_cert).unwrap();
    let sp_key_file = f._dir.path().join("sp.key");
    riauth::config::write_private(
        &sp_key_file,
        &sp_key.private_key_to_pem_pkcs8().unwrap(),
        false,
    )
    .unwrap();
    let metadata = f.core.saml_metadata("saml-app").unwrap();
    assert!(metadata.contains("WantAuthnRequestsSigned=\"true\""));
    if let Some(binary) = independent {
        verify_xml(binary, &metadata, &cert_file, f._dir.path(), false);
    }
    let sp_der = STANDARD.encode(
        X509::from_pem(sp_cert.as_bytes())
            .unwrap()
            .to_der()
            .unwrap(),
    );
    let sp_metadata = format!(
        "<md:EntityDescriptor xmlns:md=\"urn:oasis:names:tc:SAML:2.0:metadata\" xmlns:ds=\"http://www.w3.org/2000/09/xmldsig#\" entityID=\"{SP}\"><md:SPSSODescriptor protocolSupportEnumeration=\"{P}\" AuthnRequestsSigned=\"true\"><md:KeyDescriptor use=\"signing\"><ds:KeyInfo><ds:X509Data><ds:X509Certificate>{sp_der}</ds:X509Certificate></ds:X509Data></ds:KeyInfo></md:KeyDescriptor><md:AssertionConsumerService Binding=\"urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST\" Location=\"{ACS}\" index=\"7\" isDefault=\"true\"/></md:SPSSODescriptor></md:EntityDescriptor>"
    );
    let imported = riauth::saml::import_sp_metadata(&sp_metadata, SP, idp_cert.clone()).unwrap();
    assert_eq!(imported["settings"]["saml"]["acs_indices"]["7"], ACS);
    assert_eq!(
        imported["settings"]["saml"]["sp_certificates_pem"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(
        riauth::saml::import_sp_metadata(
            &sp_metadata,
            "https://wrong.example/sp",
            idp_cert.clone()
        )
        .is_err()
    );
    let metadata_file = f._dir.path().join("sp-metadata.xml");
    std::fs::write(&metadata_file, &sp_metadata).unwrap();
    let report_file = f._dir.path().join("sp-import.json");
    let imported_cli = std::process::Command::new(env!("CARGO_BIN_EXE_riauth"))
        .args(["--json", "--non-interactive", "saml", "import-sp", "--file"])
        .arg(&metadata_file)
        .args(["--entity-id", SP, "--idp-certificate"])
        .arg(&cert_file)
        .arg("--out")
        .arg(&report_file)
        .output()
        .unwrap();
    assert!(
        imported_cli.status.success(),
        "{}",
        String::from_utf8_lossy(&imported_cli.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&std::fs::read(&report_file).unwrap()).unwrap(),
        imported
    );
    assert!(f.core.saml_initiate("saml-app", None).is_err());
    let request = redirect(
        &authn("_one", "ForceAuthn=\"true\""),
        &sp_key,
        "SAMLRequest",
    );
    let (code, id, binding) = waiting(
        f.core
            .saml_start("saml-app", &request, false, None)
            .unwrap(),
    );
    assert!(
        f.core
            .saml_start("saml-app", &request, false, None)
            .is_err()
    );
    let details = f.core.browser_details(&session, &code).unwrap();
    assert_eq!(details["protocol"], "saml");
    assert_eq!(details["reauthentication_required"], true);
    assert!(
        f.core
            .browser_decide(
                &session,
                BrowserDecision {
                    code: code.clone(),
                    approve: true,
                    transaction_id: None,
                    remember: true
                }
            )
            .is_err()
    );
    let transaction = text(&details, "transaction_id");
    let fresh = text(
        &f.core
            .login_for(
                "saml-user".into(),
                PASSWORD.into(),
                None,
                Some(transaction.clone()),
            )
            .unwrap(),
        "session_token",
    );
    assert!(
        f.core
            .browser_decide(
                &other,
                BrowserDecision {
                    code: code.clone(),
                    approve: true,
                    transaction_id: Some(transaction.clone()),
                    remember: true
                }
            )
            .is_err()
    );
    let other_request = redirect(
        &authn("_parallel", "ForceAuthn=\"true\""),
        &sp_key,
        "SAMLRequest",
    );
    let (parallel, _, _) = waiting(
        f.core
            .saml_start("saml-app", &other_request, false, None)
            .unwrap(),
    );
    let other_transaction = text(
        &f.core.saml_details(&fresh, &parallel).unwrap(),
        "transaction_id",
    );
    let other_proof = text(
        &f.core
            .login_for(
                "saml-user".into(),
                PASSWORD.into(),
                None,
                Some(other_transaction.clone()),
            )
            .unwrap(),
        "session_token",
    );
    assert!(
        f.core
            .browser_decide(
                &other_proof,
                BrowserDecision {
                    code: code.clone(),
                    approve: true,
                    transaction_id: Some(other_transaction),
                    remember: false
                }
            )
            .is_err()
    );
    approve(&f, &fresh, &code, Some(transaction), true);
    assert!(f.core.saml_resume(&id, Some("wrong browser")).is_err());
    let (xml, cookies) = body(f.core.saml_resume(&id, Some(&binding)).unwrap());
    assert!(f.core.saml_resume(&id, Some(&binding)).is_err());
    let doc = roxmltree::Document::parse(&xml).unwrap();
    let root = doc.root_element();
    assert_eq!(root.attribute("InResponseTo"), Some("_one"));
    assert_eq!(root.attribute("Destination"), Some(ACS));
    let find = |name| {
        doc.descendants()
            .find(|n| n.has_tag_name((A, name)))
            .unwrap()
    };
    let name_id = find("NameID").text().unwrap().to_owned();
    let qualifier = find("NameID")
        .attribute("NameQualifier")
        .unwrap()
        .to_owned();
    assert_eq!(find("Audience").text(), Some(SP));
    assert_eq!(
        find("SubjectConfirmationData").attribute("Recipient"),
        Some(ACS)
    );
    assert_eq!(
        find("SubjectConfirmationData").attribute("InResponseTo"),
        Some("_one")
    );
    assert_eq!(
        find("AuthnContextClassRef").text(),
        Some("urn:oasis:names:tc:SAML:2.0:ac:classes:Password")
    );
    let index = find("AuthnStatement")
        .attribute("SessionIndex")
        .unwrap()
        .to_owned();
    assert!(find("AuthnStatement").attribute("AuthnInstant").is_some());
    assert!(
        doc.descendants()
            .any(|n| n.has_tag_name((A, "AttributeValue")) && n.text() == Some("Name <&\" escape"))
    );
    assert_eq!(
        doc.descendants()
            .filter(|n| n.has_tag_name(("http://www.w3.org/2000/09/xmldsig#", "Signature")))
            .count(),
        2
    );
    assert!(
        risaml::crypto::verify_signature(&xml, std::slice::from_ref(&idp_cert))
            .unwrap()
            .0
    );
    if let Some(binary) = independent {
        verify_xml(binary, &xml, &cert_file, f._dir.path(), false);
        verify_xml(binary, &xml, &cert_file, f._dir.path(), true);
    }
    let sso = cookies
        .iter()
        .find(|s| s.starts_with("riauth_sso="))
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1
        .to_owned();
    let tampered = request.replace("state+%26+exact+%2B", "changed");
    assert!(
        f.core
            .saml_start("saml-app", &tampered, false, None)
            .is_err()
    );
    let wrong = authn("_bad-acs", "").replace(ACS, "https://attacker.example/acs");
    assert!(
        f.core
            .saml_start(
                "saml-app",
                &redirect(&wrong, &sp_key, "SAMLRequest"),
                false,
                None
            )
            .is_err()
    );
    let expired = authn("_expired", "").replace(&timestamp(now()), &timestamp(now() - 400));
    assert!(
        f.core
            .saml_start(
                "saml-app",
                &redirect(&expired, &sp_key, "SAMLRequest"),
                false,
                None
            )
            .is_err()
    );
    let nested = authn("_wrapped", "").replace(
        "<samlp:NameIDPolicy",
        "<samlp:Extensions><saml:Assertion ID=\"_wrapped\"/></samlp:Extensions><samlp:NameIDPolicy",
    );
    assert!(
        f.core
            .saml_start(
                "saml-app",
                &redirect(&nested, &sp_key, "SAMLRequest"),
                false,
                None
            )
            .is_err()
    );
    let passive = redirect(
        &authn("_passive", "IsPassive=\"true\""),
        &sp_key,
        "SAMLRequest",
    );
    let (xml, _) = body(
        f.core
            .saml_start("saml-app", &passive, false, None)
            .unwrap(),
    );
    assert!(xml.contains(":NoPassive"));
    assert!(!xml.contains("<saml:Assertion"));
    let mut post_xml = risaml::crypto::construct_saml_signature(
        &authn("_post", ""),
        true,
        &risaml::crypto::keys::load_private_key(
            &String::from_utf8(sp_key.private_key_to_pem_pkcs8().unwrap()).unwrap(),
            None,
        )
        .unwrap(),
        &sp_cert,
        ALG,
        &[],
        None,
    )
    .unwrap();
    if let Some(binary) = independent {
        let input = f._dir.path().join("sp-template.xml");
        let output = f._dir.path().join("sp-signed.xml");
        std::fs::write(&input, &post_xml).unwrap();
        xmlsec(
            binary,
            &[
                "--sign",
                "--lax-key-search",
                "--privkey-pem",
                sp_key_file.to_str().unwrap(),
                "--id-attr:ID",
                "AuthnRequest",
                "--output",
                output.to_str().unwrap(),
                input.to_str().unwrap(),
            ],
        );
        post_xml = std::fs::read_to_string(output).unwrap();
    }
    let post_form = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs([
            ("SAMLRequest", STANDARD.encode(post_xml)),
            ("RelayState", "state & exact +".into()),
        ])
        .finish();
    let (xml, _) = body(
        f.core
            .saml_start("saml-app", &post_form, true, Some(&sso))
            .unwrap(),
    );
    assert!(xml.contains("InResponseTo=\"_post\""));
    let indexed = authn("_index", "").replace(
        &format!("AssertionConsumerServiceURL=\"{ACS}\""),
        "AssertionConsumerServiceIndex=\"7\"",
    );
    body(
        f.core
            .saml_start(
                "saml-app",
                &redirect(&indexed, &sp_key, "SAMLRequest"),
                false,
                Some(&sso),
            )
            .unwrap(),
    );
    let logout = |id: &str, subject: &str| {
        format!(
            "<samlp:LogoutRequest xmlns:samlp=\"{P}\" xmlns:saml=\"{A}\" ID=\"{id}\" Version=\"2.0\" IssueInstant=\"{}\" Destination=\"{ENDPOINT}\"><saml:Issuer>{SP}</saml:Issuer><saml:NameID Format=\"{}\" NameQualifier=\"{}\" SPNameQualifier=\"{SP}\">{subject}</saml:NameID><samlp:SessionIndex>{index}</samlp:SessionIndex></samlp:LogoutRequest>",
            timestamp(now()),
            NameIdFormat::Persistent.uri(),
            qualifier
        )
    };
    assert!(
        f.core
            .saml_start(
                "saml-app",
                &redirect(
                    &logout("_wrong-logout", "wrong-user"),
                    &sp_key,
                    "SAMLRequest"
                ),
                false,
                None
            )
            .is_err()
    );
    assert!(f.core.consents(&fresh).is_ok());
    let reply = f
        .core
        .saml_start(
            "saml-app",
            &redirect(&logout("_logout", &name_id), &sp_key, "SAMLRequest"),
            false,
            None,
        )
        .unwrap();
    let Reply::Redirect(target) = reply else {
        panic!("expected logout redirect")
    };
    assert!(target.starts_with("https://sp.example.test/logout?"));
    let query = url::Url::parse(&target)
        .unwrap()
        .query()
        .unwrap()
        .to_owned();
    let (signed, _) = query.rsplit_once("&Signature=").unwrap();
    let values: std::collections::HashMap<_, _> = url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect();
    let compressed = STANDARD.decode(&values["SAMLResponse"]).unwrap();
    let mut decoded = String::new();
    flate2::read::DeflateDecoder::new(compressed.as_slice())
        .read_to_string(&mut decoded)
        .unwrap();
    let logout_doc = roxmltree::Document::parse(&decoded).unwrap();
    assert!(
        logout_doc
            .root_element()
            .has_tag_name((P, "LogoutResponse"))
    );
    assert_eq!(
        logout_doc.root_element().attribute("InResponseTo"),
        Some("_logout")
    );
    let mut verifier = Verifier::new(MessageDigest::sha256(), &idp_key).unwrap();
    verifier.update(signed.as_bytes()).unwrap();
    assert!(
        verifier
            .verify(&STANDARD.decode(&values["Signature"]).unwrap())
            .unwrap()
    );
    assert!(f.core.consents(&fresh).is_err());
    // Encryption runs through the same approved browser delivery path.
    settings.encryption_certificate_pem = Some(sp_cert.clone());
    settings.idp_initiated = true;
    settings.default_relay_state = Some("state & exact +".into());
    f.core
        .update_client(
            &f.admin,
            "saml-app",
            ClientPatch {
                settings: Some(ProviderSettings {
                    saml: Some(settings),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();
    let session = text(
        &f.core
            .login("saml-user".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    let (code, id, binding) = waiting(f.core.saml_initiate("saml-app", None).unwrap());
    approve(&f, &session, &code, None, false);
    let (xml, _) = body(f.core.saml_resume(&id, Some(&binding)).unwrap());
    assert!(xml.contains("EncryptedAssertion"));
    assert!(!xml.contains("Name &lt;"));
    assert!(!xml.contains("InResponseTo="));
    assert!(
        risaml::crypto::verify_signature(&xml, std::slice::from_ref(&idp_cert))
            .unwrap()
            .0
    );
    let key = risaml::crypto::keys::load_private_key(
        &String::from_utf8(sp_key.private_key_to_pem_pkcs8().unwrap()).unwrap(),
        None,
    )
    .unwrap();
    let (_, assertion) = risaml::crypto::decrypt_assertion(&xml, &key, Default::default()).unwrap();
    assert!(assertion.contains("AuthnStatement"));
    if let Some(binary) = independent {
        verify_xml(binary, &xml, &cert_file, f._dir.path(), false);
        let input = f._dir.path().join("encrypted.xml");
        let output = f._dir.path().join("decrypted.xml");
        std::fs::write(&input, &xml).unwrap();
        xmlsec(
            binary,
            &[
                "--decrypt",
                "--lax-key-search",
                "--privkey-pem",
                sp_key_file.to_str().unwrap(),
                "--output",
                output.to_str().unwrap(),
                input.to_str().unwrap(),
            ],
        );
        let decrypted = std::fs::read_to_string(output).unwrap();
        verify_xml(binary, &decrypted, &cert_file, f._dir.path(), true);
        assert!(
            roxmltree::Document::parse(&decrypted)
                .unwrap()
                .descendants()
                .any(|n| n.has_tag_name((A, "NameID")) && n.text() == Some(&name_id))
        );
    }
    let (code, id, binding) = waiting(f.core.saml_initiate("saml-app", None).unwrap());
    approve(&f, &session, &code, None, true);
    f.core.revoke_consent(&session, "saml-app").unwrap();
    assert!(f.core.saml_resume(&id, Some(&binding)).is_err());
    let (code, id, binding) = waiting(f.core.saml_initiate("saml-app", None).unwrap());
    approve(&f, &session, &code, None, false);
    f.core
        .group_member(&f.admin, "saml-group", "saml-user", false)
        .unwrap();
    assert!(f.core.saml_resume(&id, Some(&binding)).is_err());
}
#[test]
fn saml_signed_requests_terminal_delivery_claims_encryption_logout() {
    exercise(None);
}
#[test]
#[ignore = "requires independent xmlsec1 binary in RIAUTH_TEST_XMLSEC"]
fn saml_independent_xmlsec_sign_verify_and_decrypt() {
    let path = std::env::var_os("RIAUTH_TEST_XMLSEC").expect("Set RIAUTH_TEST_XMLSEC");
    exercise(Some(Path::new(&path)));
}
#[tokio::test]
async fn trailing_slash_issuers_advertise_reachable_saml_endpoints() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    for issuer in ["http://localhost:9000/", "http://localhost:9000/auth/"] {
        let dir = tempfile::TempDir::new().unwrap();
        let core = Core::initialize(
            Config {
                data_dir: dir.path().into(),
                issuer: issuer.into(),
                ..Default::default()
            },
            NewUser {
                username: "admin".into(),
                password: PASSWORD.into(),
                email: None,
                display_name: "Administrator".into(),
                admin: true,
            },
        )
        .unwrap();
        let admin = text(
            &core.login("admin".into(), PASSWORD.into(), None).unwrap(),
            "session_token",
        );
        assert_eq!(core.config.issuer, issuer);
        assert_eq!(core.discovery()["issuer"], issuer);
        let base = issuer.trim_end_matches('/');
        let idp_keys: crypto::Keys = core.store.get("meta", "keys").unwrap().unwrap();
        let idp_key = PKey::private_key_from_pem(idp_keys.active.pem.as_bytes()).unwrap();
        let idp_cert = cert(&idp_key, "riAuth fixture");
        let sp_key = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();
        let sp_cert = cert(&sp_key, "SP fixture");
        core.create_client(
            &admin,
            NewClient {
                client_id: "saml-app".into(),
                name: "SAML fixture".into(),
                confidential: false,
                redirect_uris: vec![],
                scopes: strings(&["openid", "saml", "profile"]),
                allowed_groups: BTreeSet::new(),
                require_mfa: false,
                service: false,
                settings: ProviderSettings {
                    saml: Some(Settings {
                        sp_entity_id: SP.into(),
                        acs_urls: vec![ACS.into()],
                        acs_indices: Default::default(),
                        idp_entity_id: None,
                        idp_certificate_pem: idp_cert.clone(),
                        sp_certificates_pem: vec![sp_cert.clone()],
                        encryption_certificate_pem: None,
                        name_id_format: NameIdFormat::Persistent,
                        attributes: vec![],
                        slo_redirect_url: Some("https://sp.example.test/logout".into()),
                        slo_post_url: Some("https://sp.example.test/logout".into()),
                        idp_initiated: false,
                        default_relay_state: None,
                        assertion_ttl: 120,
                    }),
                    ..Default::default()
                },
            },
        )
        .unwrap();
        let sso = format!("{base}/saml/saml-app/sso");
        let metadata_entity = format!("{base}/saml/saml-app/metadata");
        let metadata = core.saml_metadata("saml-app").unwrap();
        assert!(metadata.contains(&format!("Location=\"{sso}\"")));
        assert!(metadata.contains(&format!("entityID=\"{metadata_entity}\"")));
        assert!(!metadata.contains("//saml/"));
        let source = riauth::source::Source {
            saml: Some(riauth::source::saml::Settings {
                slo_redirect_url: Some("https://enterprise.example.test/logout".into()),
                slo_post_url: Some("https://enterprise.example.test/logout".into()),
                signing_key: "signing".into(),
                sp_certificate_pem: idp_cert,
                idp_certificates_pem: vec![sp_cert],
                name_id_format: NameIdFormat::Persistent,
                name_attribute: None,
                email_attribute: None,
                email_verified_attribute: None,
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
        core.source_put(
            &admin,
            riauth::source::SourceInput {
                source: source.clone(),
                client_secret: None,
            },
        )
        .unwrap();
        let acs = format!("{base}/saml/sources/enterprise/acs");
        let slo = format!("{base}/saml/sources/enterprise/slo");
        assert_eq!(core.saml_source_callback_url("enterprise"), acs);
        let source_metadata = core.saml_source_metadata("enterprise").unwrap();
        assert!(source_metadata.contains(&format!("Location=\"{acs}\"")));
        assert!(source_metadata.contains(&format!("Location=\"{slo}\"")));
        assert!(!source_metadata.contains("//saml/"));
        let path_prefix = {
            let url = url::Url::parse(issuer).unwrap();
            url.path().trim_end_matches('/').to_owned()
        };
        let app = riauth::api::router(core);
        for (method, path, content_type, expected) in [
            (
                "GET",
                format!("{path_prefix}/saml/saml-app/sso"),
                None,
                StatusCode::BAD_REQUEST,
            ),
            (
                "GET",
                format!("{path_prefix}/saml/saml-app/metadata"),
                None,
                StatusCode::OK,
            ),
            (
                "POST",
                format!("{path_prefix}/saml/sources/enterprise/acs"),
                Some("application/x-www-form-urlencoded"),
                StatusCode::BAD_REQUEST,
            ),
            (
                "GET",
                format!("{path_prefix}/saml/sources/enterprise/slo"),
                None,
                StatusCode::BAD_REQUEST,
            ),
            (
                "GET",
                format!("{path_prefix}/saml/logout/placeholder-ticket-value-43chars!!"),
                None,
                StatusCode::NOT_FOUND,
            ),
        ] {
            let mut builder = Request::builder()
                .method(method)
                .uri(&path)
                .header("host", "localhost:9000");
            if let Some(content_type) = content_type {
                builder = builder.header("content-type", content_type);
            }
            let response = app
                .clone()
                .oneshot(builder.body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), expected, "{issuer} {method} {path}");
        }
        let broken = format!("{path_prefix}//saml/saml-app/sso");
        let missing = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&broken)
                    .header("host", "localhost:9000")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(missing.status(), StatusCode::NOT_FOUND, "{issuer} {broken}");
    }
}

/// A SAML application for browser sign-in tests; returns the SP signing key.
fn browser_app(f: &Fixture, require_mfa: bool, implicit_consent: bool) -> PKey<Private> {
    let idp_keys: crypto::Keys = f.core.store.get("meta", "keys").unwrap().unwrap();
    let idp_key = PKey::private_key_from_pem(idp_keys.active.pem.as_bytes()).unwrap();
    let sp_key = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();
    f.core
        .create_client(
            &f.admin,
            NewClient {
                client_id: "saml-app".into(),
                name: "SAML fixture".into(),
                confidential: false,
                redirect_uris: vec![],
                scopes: strings(&["openid", "saml", "profile"]),
                allowed_groups: BTreeSet::new(),
                require_mfa,
                service: false,
                settings: ProviderSettings {
                    saml: Some(Settings {
                        sp_entity_id: SP.into(),
                        acs_urls: vec![ACS.into()],
                        acs_indices: Default::default(),
                        idp_entity_id: None,
                        idp_certificate_pem: cert(&idp_key, "riAuth fixture"),
                        sp_certificates_pem: vec![cert(&sp_key, "SP fixture")],
                        encryption_certificate_pem: None,
                        name_id_format: NameIdFormat::Persistent,
                        attributes: vec![Attribute {
                            name: "urn:test:name".into(),
                            claim: "name".into(),
                            friendly_name: Some("Display name".into()),
                            required: true,
                        }],
                        slo_redirect_url: None,
                        slo_post_url: None,
                        idp_initiated: true,
                        default_relay_state: Some("state & exact +".into()),
                        assertion_ttl: 120,
                    }),
                    implicit_consent,
                    ..Default::default()
                },
            },
        )
        .unwrap();
    sp_key
}
fn signed_start(
    f: &Fixture,
    sp_key: &PKey<Private>,
    id: &str,
    extra: &str,
    sso: Option<&str>,
) -> Reply {
    let request = redirect(&authn(id, extra), sp_key, "SAMLRequest");
    f.core.saml_start("saml-app", &request, false, sso).unwrap()
}
fn sso_cookie(cookies: &[String]) -> Option<String> {
    cookies.iter().find_map(|c| {
        c.split(';')
            .next()?
            .strip_prefix("riauth_sso=")
            .map(str::to_owned)
    })
}
/// A browser-owned session from a portal-style sign-in; returns its SSO cookie.
fn browser_signed_in(f: &Fixture, username: &str) -> String {
    let staged = f
        .core
        .browser_password_login(username.into(), PASSWORD.into(), None, None)
        .unwrap();
    let attached = f
        .core
        .store
        .write(|tx| f.core.attach_browser_login(tx, &staged, None, None))
        .unwrap()
        .unwrap();
    sso_cookie(&attached.cookies).unwrap()
}
fn sso_session(f: &Fixture, sso: &str) -> String {
    let mapping: Value = f
        .core
        .store
        .get("browser_sessions", &digest(sso))
        .unwrap()
        .unwrap();
    text(&mapping, "session_id")
}
fn password(
    f: &Fixture,
    id: &str,
    binding: &str,
    sso: Option<&str>,
    username: &str,
) -> riauth::error::Result<BrowserReply> {
    f.core.saml_password(
        id,
        Some(binding),
        sso,
        username.into(),
        PASSWORD.into(),
        None,
    )
}
/// Signs this browser in through the interaction; returns the state and the new SSO cookie.
fn sign_in(
    f: &Fixture,
    id: &str,
    binding: &str,
    sso: Option<&str>,
    username: &str,
) -> (Value, String) {
    let reply = password(f, id, binding, sso, username).unwrap();
    assert!(reply.location.is_none() && reply.refresh.is_none());
    assert!(!reply.body.to_string().contains("ri_sso_"));
    (reply.body, sso_cookie(&reply.cookies).unwrap())
}
fn allow(
    f: &Fixture,
    id: &str,
    binding: &str,
    sso: &str,
    state: &Value,
) -> riauth::error::Result<Value> {
    let session_ref = state["session_ref"].as_str().map(str::to_owned);
    f.core
        .saml_browser_decide(id, Some(binding), Some(sso), true, true, session_ref)
}
fn pending_row(f: &Fixture, id: &str) -> Option<Value> {
    f.core.store.get("saml_requests", id).unwrap()
}
fn user_id(f: &Fixture, username: &str) -> String {
    f.core.store.get("usernames", username).unwrap().unwrap()
}
fn audited(f: &Fixture, actor: &str, action: &str, target: &str) -> bool {
    f.core
        .audit_events(&f.admin, 1000)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["actor"] == actor && e["action"] == action && e["target"] == target)
}

#[test]
fn saml_browser_password_sign_in_binds_proof_to_request() {
    let f = Fixture::new();
    browser_app(&f, false, false);
    f.user("alice");
    let (_, id, binding) = waiting(f.core.saml_initiate("saml-app", None).unwrap());
    let (_, other, _) = waiting(f.core.saml_initiate("saml-app", None).unwrap());
    let (state, sso) = sign_in(&f, &id, &binding, None, "alice");
    let holder = sso_session(&f, &sso);
    assert_eq!(state["kind"], "saml");
    assert_eq!(state["status"], "consent");
    assert_eq!(state["consent"]["required"], true);
    assert_eq!(state["account"]["username"], "alice");
    assert_eq!(state["account"]["mfa"], false);
    assert_eq!(state["pinned"], true);
    assert_eq!(state["session_ref"], signin::session_ref(&id, &holder));
    let session: Session = f.core.store.get("sessions", &holder).unwrap().unwrap();
    assert!(
        !f.core
            .store
            .read(|tx| signin::bearer_backed(tx, &session))
            .unwrap()
    );
    // The proof names this request and the session that signed in, and nothing else.
    let hash = |id: &str| {
        let row = pending_row(&f, id).unwrap();
        digest(&format!("saml\0{id}\0{}", text(&row, "client_fingerprint")))
    };
    let (this, that) = (hash(&id), hash(&other));
    let proof = text(&pending_row(&f, &id).unwrap(), "authentication");
    let stored: AuthenticationTransaction =
        f.core.store.get("authentication", &proof).unwrap().unwrap();
    assert_eq!(stored.user_id, Some(user_id(&f, "alice")));
    assert_eq!(
        stored.expires_at,
        pending_row(&f, &id).unwrap()["expires_at"]
    );
    let valid = |hash: &str, sid: &str| {
        f.core
            .store
            .read(|tx| signin::proof_valid(tx, Some(&proof), hash, sid))
            .unwrap()
    };
    assert!(valid(&this, &holder));
    assert!(!valid(&that, &holder));
    assert!(!valid(&this, "another-session"));
    assert!(pending_row(&f, &other).unwrap()["authentication"].is_null());
    // Signing in again keeps the session and replaces the proof.
    let (again, sso) = sign_in(&f, &id, &binding, Some(&sso), "alice");
    assert_eq!(sso_session(&f, &sso), holder);
    assert_eq!(again["session_ref"], state["session_ref"]);
    let replaced = text(&pending_row(&f, &id).unwrap(), "authentication");
    assert_ne!(replaced, proof);
    assert!(
        f.core
            .store
            .get::<Value>("authentication", &proof)
            .unwrap()
            .is_none()
    );
    // The decision must name the session the page showed.
    let mut moved = again.clone();
    moved["session_ref"] = json!(signin::session_ref(&other, &holder));
    let changed = allow(&f, &id, &binding, &sso, &moved).unwrap_err();
    assert_eq!(
        (changed.status.as_u16(), changed.code),
        (409, "account_changed")
    );
    let done = allow(&f, &id, &binding, &sso, &again).unwrap();
    assert_eq!(done["status"], "complete");
    assert_eq!(done["continue"], format!("/saml/resume/{id}"));
    assert!(
        f.core
            .store
            .get::<Value>("authentication", &replaced)
            .unwrap()
            .is_none()
    );
    assert!(audited(
        &f,
        &user_id(&f, "alice"),
        "saml.approve",
        "saml-app"
    ));
    let (xml, _) = body(
        f.core
            .saml_resume_with(&id, Some(&binding), Some(&sso))
            .unwrap(),
    );
    assert!(xml.contains(":status:Success\""));
    assert!(xml.contains("urn:oasis:names:tc:SAML:2.0:ac:classes:Password"));
    assert!(pending_row(&f, &id).is_none());
}

#[test]
fn saml_force_authn_requires_proof_from_interaction() {
    let f = Fixture::new();
    let sp_key = browser_app(&f, false, false);
    f.user("alice");
    f.user("bob");
    let sso = browser_signed_in(&f, "alice");
    let sid = sso_session(&f, &sso);
    let force = "ForceAuthn=\"true\"";
    let (_, first, first_binding) = waiting(signed_start(&f, &sp_key, "_first", force, Some(&sso)));
    let (_, second, second_binding) =
        waiting(signed_start(&f, &sp_key, "_second", force, Some(&sso)));
    let state = f
        .core
        .saml_state(&first, Some(&first_binding), Some(&sso))
        .unwrap();
    assert_eq!(state["status"], "authenticate");
    assert_eq!(state["reason"], "force_authn");
    assert_eq!(state["pinned"], true);
    assert_eq!(state["account"]["username"], "alice");
    // The existing session alone cannot approve.
    let required = allow(&f, &first, &first_binding, &sso, &state).unwrap_err();
    assert_eq!(
        (required.status.as_u16(), required.code),
        (400, "login_required")
    );
    // The request is pinned to the signed-in account.
    assert_eq!(
        password(&f, &first, &first_binding, Some(&sso), "bob")
            .err()
            .unwrap()
            .code,
        "invalid_credentials"
    );
    let kept: Session = f.core.store.get("sessions", &sid).unwrap().unwrap();
    assert!(!kept.revoked);
    assert_eq!(sso_session(&f, &sso), sid);
    assert_eq!(
        f.core
            .saml_passkey_start(&first, Some(&first_binding), Some(&sso))
            .unwrap_err()
            .code,
        "no_passkey"
    );
    // A sign-in for another request proves nothing for this one.
    let (other_state, sso) = sign_in(&f, &second, &second_binding, Some(&sso), "alice");
    assert_eq!(other_state["status"], "consent");
    assert_eq!(sso_session(&f, &sso), sid);
    let state = f
        .core
        .saml_state(&first, Some(&first_binding), Some(&sso))
        .unwrap();
    assert_eq!(state["status"], "authenticate");
    assert_eq!(
        allow(&f, &first, &first_binding, &sso, &state)
            .unwrap_err()
            .code,
        "login_required"
    );
    let (state, sso) = sign_in(&f, &first, &first_binding, Some(&sso), "alice");
    assert_eq!(state["status"], "consent");
    assert_eq!(
        sso_session(&f, &sso),
        sid,
        "the same user keeps the session"
    );
    assert_eq!(
        allow(&f, &first, &first_binding, &sso, &state).unwrap()["status"],
        "complete"
    );
    let (xml, _) = body(
        f.core
            .saml_resume_with(&first, Some(&first_binding), Some(&sso))
            .unwrap(),
    );
    assert!(xml.contains("InResponseTo=\"_first\""));
    assert!(xml.contains(":status:Success\""));
}

#[test]
fn saml_implicit_consent_issues_silently() {
    let f = Fixture::new();
    let sp_key = browser_app(&f, false, true);
    f.user("alice");
    let (_, id, binding) = waiting(f.core.saml_initiate("saml-app", None).unwrap());
    let before = f.core.saml_state(&id, Some(&binding), None).unwrap();
    assert_eq!(before["consent"]["required"], false);
    let (state, sso) = sign_in(&f, &id, &binding, None, "alice");
    assert_eq!(
        state["status"], "complete",
        "no consent screen after sign-in"
    );
    assert_eq!(state["continue"], format!("/saml/resume/{id}"));
    assert!(audited(
        &f,
        &user_id(&f, "alice"),
        "saml.approve",
        "saml-app"
    ));
    let (xml, cookies) = body(
        f.core
            .saml_resume_with(&id, Some(&binding), Some(&sso))
            .unwrap(),
    );
    assert!(xml.contains(":status:Success\""));
    assert_eq!(sso_cookie(&cookies), None);
    // Nothing is remembered, yet the next request from this browser is answered at once.
    assert!(
        f.core
            .store
            .list::<Value>("saml_consents")
            .unwrap()
            .is_empty()
    );
    body(f.core.saml_initiate("saml-app", Some(&sso)).unwrap());
    // ForceAuthn still asks this browser to sign in again.
    let (_, forced, forced_binding) = waiting(signed_start(
        &f,
        &sp_key,
        "_forced",
        "ForceAuthn=\"true\"",
        Some(&sso),
    ));
    let state = f
        .core
        .saml_state(&forced, Some(&forced_binding), Some(&sso))
        .unwrap();
    assert_eq!(state["reason"], "force_authn");
}

#[test]
fn saml_cancel_issues_request_denied() {
    let f = Fixture::new();
    let sp_key = browser_app(&f, false, false);
    let alice = f.user("alice");
    let (code, id, binding) = waiting(signed_start(&f, &sp_key, "_cancel", "", None));
    // Approving needs a signed-in browser; cancelling needs only the binding cookie.
    for (binding, approve) in [(binding.as_str(), true), ("another browser", false)] {
        let error = f
            .core
            .saml_browser_decide(&id, Some(binding), None, approve, false, None)
            .unwrap_err();
        assert_eq!(error.code, "invalid_token");
    }
    let state = f
        .core
        .saml_browser_decide(&id, Some(&binding), None, false, false, None)
        .unwrap();
    assert_eq!(state["status"], "complete");
    assert_eq!(state["continue"], format!("/saml/resume/{id}"));
    assert!(audited(&f, "anonymous", "saml.deny", "saml-app"));
    for error in [
        f.core
            .saml_browser_decide(&id, Some(&binding), None, false, false, None)
            .unwrap_err(),
        password(&f, &id, &binding, None, "alice").err().unwrap(),
        f.core
            .saml_passkey_start(&id, Some(&binding), None)
            .unwrap_err(),
    ] {
        assert_eq!(
            (error.status.as_u16(), error.code),
            (409, "request_decided")
        );
    }
    assert!(f.core.saml_details(&alice, &code).is_err());
    let terminal = BrowserDecision {
        code,
        approve: true,
        transaction_id: None,
        remember: false,
    };
    assert!(f.core.browser_decide(&alice, terminal).is_err());
    let (xml, cookies) = body(f.core.saml_resume_with(&id, Some(&binding), None).unwrap());
    assert!(xml.contains(":status:RequestDenied\""));
    assert!(xml.contains("InResponseTo=\"_cancel\""));
    assert!(!xml.contains("<saml:Assertion"));
    assert_eq!(cookies.len(), 1);
    assert!(cookies[0].starts_with("riauth_saml=;"));
    assert!(pending_row(&f, &id).is_none());
    let gone = f
        .core
        .saml_resume_with(&id, Some(&binding), None)
        .err()
        .unwrap();
    assert_eq!(gone.status.as_u16(), 404);
}

#[test]
fn saml_browser_approval_is_delivered_only_to_the_approving_browser() {
    let f = Fixture::new();
    browser_app(&f, false, false);
    f.user("alice");
    f.user("bob");
    let bob = browser_signed_in(&f, "bob");
    let bob_sid = sso_session(&f, &bob);
    // The consent screen approves the first request and remembers it; the second one is
    // approved at sign-in. Neither goes to a planted binding cookie, alone or with another
    // browser's session, and the request waits for its own browser.
    let mut sso = None;
    for automatic in [false, true] {
        let (_, id, binding) = waiting(f.core.saml_initiate("saml-app", None).unwrap());
        let (state, signed_in) = sign_in(&f, &id, &binding, sso.as_deref(), "alice");
        if automatic {
            assert_eq!(state["status"], "complete");
        } else {
            allow(&f, &id, &binding, &signed_in, &state).unwrap();
        }
        for stranger in [None, Some(bob.as_str())] {
            let refused = f
                .core
                .saml_resume_with(&id, Some(&binding), stranger)
                .err()
                .unwrap();
            assert_eq!(refused.status.as_u16(), 401);
        }
        assert!(pending_row(&f, &id).is_some());
        let (xml, _) = body(
            f.core
                .saml_resume_with(&id, Some(&binding), Some(&signed_in))
                .unwrap(),
        );
        assert!(xml.contains(":status:Success\""));
        sso = Some(signed_in);
    }
    let session: Session = f.core.store.get("sessions", &bob_sid).unwrap().unwrap();
    assert!(!session.revoked, "the stranger's session is untouched");
}

#[test]
fn saml_resume_reuses_existing_browser_session_cookie() {
    let f = Fixture::new();
    browser_app(&f, false, false);
    f.user("alice");
    let (_, id, binding) = waiting(f.core.saml_initiate("saml-app", None).unwrap());
    let (state, sso) = sign_in(&f, &id, &binding, None, "alice");
    let holder = sso_session(&f, &sso);
    allow(&f, &id, &binding, &sso, &state).unwrap();
    let (xml, cookies) = body(
        f.core
            .saml_resume_with(&id, Some(&binding), Some(&sso))
            .unwrap(),
    );
    assert!(xml.contains(":status:Success\""));
    assert_eq!(
        cookies.len(),
        1,
        "only the binding cookie is cleared: {cookies:?}"
    );
    assert!(cookies[0].starts_with("riauth_saml=;"));
    let mapping: Value = f
        .core
        .store
        .get("browser_sessions", &digest(&sso))
        .unwrap()
        .unwrap();
    assert_eq!(mapping["session_id"], holder);
    assert!(mapping.get("rotated").is_none());
    assert_eq!(
        f.core.portal_apps(Some(&sso)).unwrap()["user"]["username"],
        "alice"
    );
    // The remembered consent lets this browser through at once next time.
    body(f.core.saml_initiate("saml-app", Some(&sso)).unwrap());
}

#[test]
fn saml_terminal_approval_revokes_displaced_browser_owned_session() {
    let f = Fixture::new();
    browser_app(&f, false, false);
    f.client("app", false);
    let alice = f.user("alice");
    let bob = f.user("bob");
    let bob_sso = browser_signed_in(&f, "bob");
    let bob_sid = sso_session(&f, &bob_sso);
    // An OIDC sign-in on the browser session, so revoking it must reach the RP.
    let verifier = crypto::random_token("");
    let bob_session: Session = f.core.store.get("sessions", &bob_sid).unwrap().unwrap();
    let location = f
        .core
        .store
        .write(|tx| {
            f.core.authorize_session_proof(
                tx,
                bob_session,
                f.request("app", &verifier),
                false,
                None,
            )
        })
        .unwrap();
    let code = url::Url::parse(&location)
        .unwrap()
        .query_pairs()
        .find(|(k, _)| k == "code")
        .unwrap()
        .1
        .into_owned();
    f.core
        .token(TokenRequest {
            grant_type: "authorization_code".into(),
            client_id: Some("app".into()),
            code: Some(code),
            redirect_uri: Some("http://localhost:7777/callback?existing=1".into()),
            code_verifier: Some(verifier),
            ..Default::default()
        })
        .unwrap();
    let (code, id, binding) = waiting(f.core.saml_initiate("saml-app", Some(&bob_sso)).unwrap());
    let state = f
        .core
        .saml_state(&id, Some(&binding), Some(&bob_sso))
        .unwrap();
    assert_eq!(state["account"]["username"], "bob");
    approve(&f, &alice, &code, None, false);
    let (xml, cookies) = body(
        f.core
            .saml_resume_with(&id, Some(&binding), Some(&bob_sso))
            .unwrap(),
    );
    assert!(xml.contains(":status:Success\""));
    let alice_sso = sso_cookie(&cookies).unwrap();
    let alice_sid = text(&f.core.me(&alice).unwrap(), "session_id");
    assert_eq!(sso_session(&f, &alice_sso), alice_sid);
    let displaced: Session = f.core.store.get("sessions", &bob_sid).unwrap().unwrap();
    assert!(displaced.revoked);
    assert!(audited(
        &f,
        &user_id(&f, "alice"),
        "session.revoke",
        &bob_sid
    ));
    let rp: Vec<riauth::logout::RpSession> = f
        .core
        .store
        .list::<riauth::logout::RpSession>("rp_sessions")
        .unwrap()
        .into_iter()
        .map(|(_, rp)| rp)
        .filter(|rp| rp.session_id == bob_sid)
        .collect();
    assert!(
        !rp.is_empty() && rp.iter().all(|rp| rp.ended),
        "logout queued"
    );
    assert!(f.core.portal_apps(Some(&bob_sso)).is_err());
    // A terminal session shown in the browser only loses the mapping.
    let (code, id, binding) = waiting(f.core.saml_initiate("saml-app", Some(&alice_sso)).unwrap());
    approve(&f, &bob, &code, None, false);
    let (_, cookies) = body(
        f.core
            .saml_resume_with(&id, Some(&binding), Some(&alice_sso))
            .unwrap(),
    );
    let bob_terminal = text(&f.core.me(&bob).unwrap(), "session_id");
    assert_eq!(
        sso_session(&f, &sso_cookie(&cookies).unwrap()),
        bob_terminal
    );
    assert!(f.core.me(&alice).is_ok());
    assert!(f.core.portal_apps(Some(&alice_sso)).is_err());
}

#[test]
fn saml_terminal_approval_requires_recent_authentication() {
    let f = Fixture::new();
    browser_app(&f, false, false);
    let alice = f.user("alice");
    let context = riauth::context::RequestContext {
        client_ip: Some("192.0.2.7".parse().unwrap()),
        user_agent: Some("Example Browser/1.0".into()),
        ..Default::default()
    };
    let started = riauth::context::scope(Some(context), || f.core.saml_initiate("saml-app", None));
    let (code, id, binding) = waiting(started.unwrap());
    let details = f.core.saml_details(&alice, &code).unwrap();
    assert_eq!(details["reauthentication_required"], false);
    let requester = &details["requested_from"];
    assert_eq!(requester["ip"], "192.0.2.7");
    assert_eq!(requester["user_agent"], "Example Browser/1.0");
    assert!(requester["at"].as_u64().unwrap() + 5 >= now());
    // Past 240 s the CLI is asked to sign in again; past 300 s an approval fails.
    age_session_and_grant_timestamps(&f.core, 241);
    let details = f.core.saml_details(&alice, &code).unwrap();
    assert_eq!(details["reauthentication_required"], true);
    age_session_and_grant_timestamps(&f.core, 60);
    let decision = BrowserDecision {
        code: code.clone(),
        approve: true,
        transaction_id: None,
        remember: false,
    };
    let stale = f.core.browser_decide(&alice, decision).unwrap_err();
    assert_eq!((stale.status.as_u16(), stale.code), (400, "login_required"));
    let fresh = text(
        &f.core.login("alice".into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    approve(&f, &fresh, &code, None, false);
    let (xml, _) = body(f.core.saml_resume_with(&id, Some(&binding), None).unwrap());
    assert!(xml.contains(":status:Success\""));
}

#[test]
fn saml_state_reports_300_second_expiry_and_expired_interaction() {
    let f = Fixture::new();
    browser_app(&f, false, false);
    let before = now();
    let reply = f.core.saml_initiate("saml-app", None).unwrap();
    let Reply::Waiting(handoff) = &reply else {
        panic!("expected terminal handoff")
    };
    let resume = text(&handoff.body, "resume_uri");
    let (code, id, binding) = waiting(reply);
    assert_eq!(resume, format!("/saml/resume/{id}"));
    let state = f.core.saml_state(&id, Some(&binding), None).unwrap();
    assert_eq!(state["kind"], "saml");
    assert_eq!(state["status"], "authenticate");
    assert_eq!(state["reason"], "sign_in");
    let expires = state["expires_at"].as_u64().unwrap();
    assert!((before + 300..=now() + 300).contains(&expires));
    assert_eq!(pending_row(&f, &id).unwrap()["expires_at"], expires);
    assert_eq!(
        state["application"],
        json!({"client_id": "saml-app", "name": "SAML fixture", "host": "sp.example.test"})
    );
    assert_eq!(
        state["terminal"],
        json!({"user_code": code, "issuer": f.core.config.issuer})
    );
    assert_eq!(
        state["requirements"],
        json!({"mfa": false, "browser": true})
    );
    assert_eq!(state["consent"]["required"], true);
    assert!(state["consent"]["scopes"].is_null());
    assert_eq!(state["consent"]["attributes"][0]["name"], "urn:test:name");
    assert_eq!(state["pinned"], false);
    for key in ["account", "session_ref", "continue", "error", "logout"] {
        assert!(state[key].is_null(), "{key}");
    }
    for binding in [None, Some("another browser")] {
        let error = f.core.saml_state(&id, binding, None).unwrap_err();
        assert_eq!((error.status.as_u16(), error.code), (401, "invalid_token"));
    }
    // A changed client configuration makes the request unusable.
    f.core
        .update_client(
            &f.admin,
            "saml-app",
            ClientPatch {
                name: Some("Renamed".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let state = f.core.saml_state(&id, Some(&binding), None).unwrap();
    assert_eq!(state["status"], "unavailable");
    assert_eq!(state["error"], "invalid_request");
    // At expiry every interaction endpoint says so.
    f.core
        .store
        .write(|tx| {
            let mut row: Value = tx.get("saml_requests", &id)?.unwrap();
            row["expires_at"] = json!(now() - 1);
            tx.put("saml_requests", &id, &row)
        })
        .unwrap();
    for error in [
        f.core.saml_state(&id, Some(&binding), None).unwrap_err(),
        password(&f, &id, &binding, None, "alice").err().unwrap(),
        f.core
            .saml_passkey_start(&id, Some(&binding), None)
            .unwrap_err(),
        f.core
            .saml_browser_decide(&id, Some(&binding), None, false, false, None)
            .unwrap_err(),
    ] {
        assert_eq!(
            (error.status.as_u16(), error.code),
            (404, "interaction_expired")
        );
    }
}

#[test]
fn saml_step_up_refuses_password_and_accepts_pinned_passkey() {
    use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};
    let f = Fixture::new();
    browser_app(&f, true, false);
    let alice = f.user("alice");
    let origin = url::Url::parse(&f.core.config.issuer).unwrap();
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let start = f
        .core
        .passkey_register_start(&alice, "Test key".into())
        .unwrap();
    let mut options = start["public_key"].clone();
    options["publicKey"]["authenticatorSelection"]["requireResidentKey"] = json!(false);
    let registration = authenticator
        .do_registration(origin.clone(), serde_json::from_value(options).unwrap())
        .unwrap();
    f.core
        .passkey_register_finish(&alice, &text(&start, "ceremony"), registration)
        .unwrap();
    let (_, id, binding) = waiting(f.core.saml_initiate("saml-app", None).unwrap());
    let state = f.core.saml_state(&id, Some(&binding), None).unwrap();
    assert_eq!(state["requirements"], json!({"mfa": true, "browser": true}));
    // A password alone never becomes a session for this client.
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    let refused = password(&f, &id, &binding, None, "alice").err().unwrap();
    assert_eq!(
        (refused.status.as_u16(), refused.code),
        (403, "unmet_authentication_requirements")
    );
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    assert!(
        f.core
            .store
            .list::<Value>("browser_logins")
            .unwrap()
            .is_empty()
    );
    // Without a session the browser may offer any discoverable passkey.
    let open = f
        .core
        .saml_passkey_start(&id, Some(&binding), None)
        .unwrap();
    assert_eq!(
        open["public_key"]["publicKey"]["allowCredentials"],
        json!([])
    );
    // A password session in this browser pins the account and asks for a step up.
    let sso = browser_signed_in(&f, "alice");
    let sid = sso_session(&f, &sso);
    let state = f.core.saml_state(&id, Some(&binding), Some(&sso)).unwrap();
    assert_eq!(state["status"], "authenticate");
    assert_eq!(state["reason"], "step_up");
    let start = f
        .core
        .saml_passkey_start(&id, Some(&binding), Some(&sso))
        .unwrap();
    let proof = authenticator
        .do_authentication(
            origin,
            serde_json::from_value(start["public_key"].clone()).unwrap(),
        )
        .unwrap();
    let reply = f
        .core
        .saml_passkey_finish(
            &id,
            Some(&binding),
            Some(&sso),
            &text(&start, "ceremony"),
            proof,
        )
        .unwrap();
    let sso = sso_cookie(&reply.cookies).unwrap();
    assert_eq!(sso_session(&f, &sso), sid, "the step-up keeps the session");
    assert_eq!(reply.body["status"], "consent");
    assert_eq!(reply.body["account"]["mfa"], true);
    assert_eq!(
        allow(&f, &id, &binding, &sso, &reply.body).unwrap()["status"],
        "complete"
    );
    let (xml, _) = body(
        f.core
            .saml_resume_with(&id, Some(&binding), Some(&sso))
            .unwrap(),
    );
    assert!(xml.contains("urn:riauth:acr:mfa"));
}

#[tokio::test]
async fn saml_browser_interaction_over_http() {
    use axum::{
        body::Body,
        http::{HeaderMap, Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    let f = Fixture::new();
    browser_app(&f, false, false);
    f.user("alice");
    let app = riauth::api::router(f.core.clone());
    let send = |request: Request<Body>| {
        let app = app.clone();
        async move {
            let response = app.oneshot(request).await.unwrap();
            let (status, headers) = (response.status(), response.headers().clone());
            let body = response.into_body().collect().await.unwrap().to_bytes();
            (status, headers, String::from_utf8_lossy(&body).into_owned())
        }
    };
    let cookie = |headers: &HeaderMap, name: &str| {
        headers
            .get_all("set-cookie")
            .iter()
            .map(|v| v.to_str().unwrap().split(';').next().unwrap().to_owned())
            .find_map(|c| c.strip_prefix(&format!("{name}=")).map(str::to_owned))
    };
    let get = |uri: &str, accept: &str, cookies: &str| {
        Request::get(uri)
            .header("accept", accept)
            .header("cookie", cookies)
            .body(Body::empty())
            .unwrap()
    };
    let post = |uri: &str, cookies: &str, body: Value| {
        Request::post(uri)
            .header("origin", "http://localhost:9000")
            .header("x-riauth-portal", "1")
            .header("sec-fetch-site", "same-origin")
            .header("content-type", "application/json")
            .header("cookie", cookies)
            .body(Body::from(body.to_string()))
            .unwrap()
    };
    // IdP-initiated, as a browser navigates: the request continues on its resume page.
    let (status, headers, _) = send(get("/saml/saml-app/init", "text/html", "")).await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    assert_eq!(headers["vary"], "Accept");
    let id = headers["location"]
        .to_str()
        .unwrap()
        .strip_prefix("http://localhost:9000/saml/resume/")
        .unwrap()
        .to_owned();
    let binding = format!("riauth_saml={}", cookie(&headers, "riauth_saml").unwrap());
    let code = text(&pending_row(&f, &id).unwrap(), "code");
    let (status, headers, page) =
        send(get(&format!("/saml/resume/{id}"), "text/html", &binding)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(headers.get("refresh").is_none());
    assert!(headers.get("cross-origin-opener-policy").is_none());
    assert!(page.contains(&format!("<code>riauth request approve {code}</code>")));
    let (_, _, state) = send(get(
        &format!("/saml/resume/{id}/state"),
        "application/json",
        &binding,
    ))
    .await;
    let state: Value = serde_json::from_str(&state).unwrap();
    assert_eq!(state["kind"], "saml");
    assert_eq!(state["status"], "authenticate");
    let (status, headers, signed_in) = send(post(
        &format!("/saml/resume/{id}/password"),
        &binding,
        json!({"username": "alice", "password": PASSWORD}),
    ))
    .await;
    assert_eq!(status, StatusCode::OK, "{signed_in}");
    let signed_in: Value = serde_json::from_str(&signed_in).unwrap();
    assert_eq!(signed_in["status"], "consent");
    assert_eq!(
        signed_in["consent"]["attributes"][0]["friendly_name"],
        "Display name"
    );
    let cookies = format!(
        "{binding}; riauth_sso={}",
        cookie(&headers, "riauth_sso").unwrap()
    );
    let (status, _, done) = send(post(
        &format!("/saml/resume/{id}/decision"),
        &cookies,
        json!({"approve": true, "remember": false, "session_ref": signed_in["session_ref"]}),
    ))
    .await;
    assert_eq!(status, StatusCode::OK, "{done}");
    let done: Value = serde_json::from_str(&done).unwrap();
    assert_eq!(done["status"], "complete");
    assert_eq!(done["continue"], format!("/saml/resume/{id}"));
    // The page continues to its resume path, which posts the response to the SP.
    let (status, headers, form) =
        send(get(&format!("/saml/resume/{id}"), "text/html", &cookies)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(form.contains(&format!("action=\"{ACS}\"")));
    assert_eq!(cookie(&headers, "riauth_saml").as_deref(), Some(""));
    let response = form
        .split("name=\"SAMLResponse\" value=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap();
    let xml = String::from_utf8(STANDARD.decode(response).unwrap()).unwrap();
    assert!(xml.contains(":status:Success\""));
    assert!(pending_row(&f, &id).is_none());
    let (status, _, _) = send(get(&format!("/saml/resume/{id}"), "text/html", &cookies)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
