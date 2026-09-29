//! Run with scripts/test-saml-sp.sh against a locally compiled GNU Lasso helper.
#![cfg(feature = "platform")]
mod common;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use common::{Fixture, PASSWORD, strings, text};
use openssl::{
    asn1::Asn1Time,
    hash::MessageDigest,
    pkey::{PKey, Private},
    rsa::Rsa,
    x509::{X509, X509NameBuilder},
};
use riauth::{
    browser::BrowserDecision,
    model::{NewClient, ProviderSettings},
    saml::{NameIdFormat, Reply, Settings},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const SP: &str = "https://sp.example.test/metadata";
const ACS: &str = "https://sp.example.test/acs";
const CLIENT: &str = "lasso-sp";
const RSA256: &str = "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256";

fn cert(key: &PKey<Private>, name: &str) -> String {
    let mut subject = X509NameBuilder::new().unwrap();
    subject.append_entry_by_text("CN", name).unwrap();
    let subject = subject.build();
    let mut builder = X509::builder().unwrap();
    builder.set_version(2).unwrap();
    builder
        .set_serial_number(
            &openssl::bn::BigNum::from_u32(1)
                .unwrap()
                .to_asn1_integer()
                .unwrap(),
        )
        .unwrap();
    builder.set_subject_name(&subject).unwrap();
    builder.set_issuer_name(&subject).unwrap();
    builder.set_pubkey(key).unwrap();
    builder
        .set_not_before(&Asn1Time::days_from_now(0).unwrap())
        .unwrap();
    builder
        .set_not_after(&Asn1Time::days_from_now(1).unwrap())
        .unwrap();
    builder.sign(key, MessageDigest::sha256()).unwrap();
    String::from_utf8(builder.build().to_pem().unwrap()).unwrap()
}

fn pem_body(pem: &str) -> String {
    pem.lines()
        .filter(|line| !line.starts_with("-----"))
        .collect()
}

fn write_private(path: &Path, data: &[u8]) {
    fs::write(path, data).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
}

fn show(output: &Output) -> String {
    format!(
        "exit={:?}\nstdout={}\nstderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn run(bin: &Path, args: &[String]) -> Output {
    Command::new(bin)
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("lasso helper: {error}"))
}

fn fields(output: &Output) -> BTreeMap<String, String> {
    assert!(output.status.success(), "{}", show(output));
    let mut parsed = BTreeMap::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let (key, value) = line
            .split_once(": ")
            .unwrap_or_else(|| panic!("helper line: {line}"));
        assert!(parsed.insert(key.to_string(), value.to_string()).is_none());
    }
    parsed
}

fn signature_reject(output: &Output) {
    assert!(!output.status.success(), "{}", show(output));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("(-111)"),
        "expected signature verification failure {}",
        show(output)
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("name_id:"),
        "{}",
        show(output)
    );
}

fn lifetime_reject(output: &Output) {
    assert!(!output.status.success(), "{}", show(output));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("lasso lifetime: assertion lifetime is not valid (1)"),
        "expected Lasso conditions lifetime rejection {}",
        show(output)
    );
    assert!(
        !stderr.contains("(-111)") && !stderr.contains("lasso audience:"),
        "lifetime rejection was another Lasso failure {}",
        show(output)
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("name_id:"),
        "{}",
        show(output)
    );
}

fn without_signatures(xml: &str) -> String {
    let mut rest = xml;
    let mut stripped = String::new();
    while let Some(start) = rest.find("<ds:Signature") {
        stripped.push_str(&rest[..start]);
        let end = rest[start..]
            .find("</ds:Signature>")
            .expect("signature end")
            + start
            + "</ds:Signature>".len();
        rest = &rest[end..];
    }
    stripped.push_str(rest);
    assert!(!stripped.contains("<ds:Signature"), "signature remained");
    stripped
}

fn attribute<'a>(tag: &'a str, name: &str) -> &'a str {
    let key = format!("{name}=\"");
    let start = tag.find(&key).unwrap_or_else(|| panic!("{name}")) + key.len();
    let end = tag[start..]
        .find('"')
        .unwrap_or_else(|| panic!("{name} end"));
    &tag[start..start + end]
}

fn conditions_range(xml: &str) -> (usize, usize) {
    let start = xml.find("<saml:Conditions ").expect("conditions");
    let end = start + xml[start..].find('>').expect("conditions tag");
    (start, end)
}

/// Move Conditions NotOnOrAfter back to NotBefore. SubjectConfirmationData keeps
/// the original expiry. The caller re-signs so Lasso's signature check still passes.
fn expire_conditions(xml: &str) -> String {
    let (start, end) = conditions_range(xml);
    let tag = &xml[start..end];
    let not_before = attribute(tag, "NotBefore").to_string();
    let not_after = attribute(tag, "NotOnOrAfter").to_string();
    assert_ne!(not_before, not_after);
    assert_eq!(not_before.len(), not_after.len());
    let at =
        start + tag.find("NotOnOrAfter=\"").expect("conditions expiry") + "NotOnOrAfter=\"".len();
    let mut expired = xml.to_string();
    expired.replace_range(at..at + not_after.len(), &not_before);
    let (start, end) = conditions_range(&expired);
    assert_eq!(attribute(&expired[start..end], "NotOnOrAfter"), not_before);
    assert!(expired.contains(&format!("NotOnOrAfter=\"{not_after}\"")));
    expired
}

fn recipient_reject(output: &Output) {
    assert!(!output.status.success(), "{}", show(output));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(
            "helper recipient: assertion Recipient does not match the SP assertion consumer service"
        ),
        "expected helper Recipient rejection {}",
        show(output)
    );
    assert!(
        !stderr.contains("(-111)")
            && !stderr.contains("lasso audience:")
            && !stderr.contains("lasso lifetime:"),
        "recipient rejection was another check {}",
        show(output)
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("name_id:"),
        "{}",
        show(output)
    );
}

const OTHER_ACS: &str = "https://sp.example.test/bad";

/// Replace only SubjectConfirmationData Recipient. Audience, Conditions, and
/// Destination stay as riAuth signed them. The caller re-signs.
fn retarget_recipient(xml: &str) -> String {
    let marker = "<saml:SubjectConfirmationData ";
    let start = xml.find(marker).expect("subject confirmation");
    let end = start + xml[start..].find('>').expect("subject confirmation tag");
    let tag = &xml[start..end];
    let recipient = attribute(tag, "Recipient");
    assert_eq!(recipient, ACS);
    assert_eq!(recipient.len(), OTHER_ACS.len());
    let at = start + tag.find("Recipient=\"").expect("recipient") + "Recipient=\"".len();
    let mut retargeted = xml.to_string();
    retargeted.replace_range(at..at + recipient.len(), OTHER_ACS);
    assert!(retargeted.contains(&format!("Recipient=\"{OTHER_ACS}\"")));
    assert!(!retargeted.contains(&format!("Recipient=\"{ACS}\"")));
    assert!(retargeted.contains(&format!("Destination=\"{ACS}\"")));
    assert!(retargeted.contains(&format!(">{SP}<")));
    retargeted
}

fn resign(xml: &str, pem: &str, certificate: &str) -> String {
    let key = risaml::crypto::keys::load_private_key(pem, None).expect("IdP signing key");
    let assertion =
        risaml::crypto::construct_saml_signature(xml, false, &key, certificate, RSA256, &[], None)
            .expect("assertion signature");
    risaml::crypto::construct_saml_signature(&assertion, true, &key, certificate, RSA256, &[], None)
        .expect("response signature")
}

fn sp_metadata(certificate: &str) -> String {
    format!(
        r#"<EntityDescriptor xmlns="urn:oasis:names:tc:SAML:2.0:metadata" entityID="{SP}"><SPSSODescriptor protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol" AuthnRequestsSigned="true" WantAssertionsSigned="true"><KeyDescriptor use="signing"><KeyInfo xmlns="http://www.w3.org/2000/09/xmldsig#"><X509Data><X509Certificate>{}</X509Certificate></X509Data></KeyInfo></KeyDescriptor><NameIDFormat>{}</NameIDFormat><AssertionConsumerService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST" Location="{ACS}" index="0" isDefault="true"/></SPSSODescriptor></EntityDescriptor>"#,
        pem_body(certificate),
        NameIdFormat::Persistent.uri()
    )
}

fn replace_certificates(metadata: &str, certificate: &str) -> String {
    let body = pem_body(certificate);
    let open = "<ds:X509Certificate>";
    let close = "</ds:X509Certificate>";
    let mut xml = metadata.to_string();
    let mut replaced = 0;
    let mut search = 0;
    while let Some(relative) = xml[search..].find(open) {
        let start = search + relative + open.len();
        let end = start
            + xml[start..]
                .find(close)
                .expect("IdP metadata certificate end");
        xml.replace_range(start..end, &body);
        replaced += 1;
        search = start + body.len() + close.len();
    }
    assert!(replaced >= 1, "IdP metadata certificate");
    xml
}

fn waiting(reply: Reply) -> (String, String, String) {
    let Reply::Waiting(reply) = reply else {
        panic!("expected a pending SAML request");
    };
    let code = reply.body["user_code"]
        .as_str()
        .expect("user code")
        .to_string();
    let id = reply
        .refresh
        .expect("resume refresh")
        .rsplit('/')
        .next()
        .expect("request id")
        .to_string();
    let cookie = reply
        .cookies
        .iter()
        .find(|cookie| cookie.starts_with("riauth_saml="))
        .expect("saml browser cookie")
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1
        .to_string();
    (code, id, cookie)
}

fn response_xml(reply: Reply) -> (String, String) {
    let Reply::Post { target, fields, .. } = reply else {
        panic!("expected a SAML POST response");
    };
    assert_eq!(target, ACS);
    let mut xml = None;
    let mut relay = None;
    for (name, value) in fields {
        match name.as_str() {
            "SAMLResponse" => xml = Some(value),
            "RelayState" => relay = Some(value),
            _ => panic!("unexpected POST field {name}"),
        }
    }
    (xml.expect("SAMLResponse"), relay.expect("RelayState"))
}

#[ignore = "requires GNU Lasso via RIAUTH_TEST_LASSO_SP from scripts/test-saml-sp.sh"]
#[test]
fn lasso_signed_redirect_request_post_response_signature_and_metadata_key() {
    let bin =
        PathBuf::from(std::env::var("RIAUTH_TEST_LASSO_SP").expect("Run scripts/test-saml-sp.sh"));
    let fixture = Fixture::new();
    let mut session = fixture.user("lasso-user");
    let idp_keys: riauth::crypto::Keys = fixture.core.store.get("meta", "keys").unwrap().unwrap();
    let idp_key = PKey::private_key_from_pem(idp_keys.active.pem.as_bytes()).unwrap();
    let idp_cert = cert(&idp_key, "riAuth fixture");
    let sp_key = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();
    let sp_cert = cert(&sp_key, "Lasso fixture");
    let wrong_key = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();
    let wrong_cert = cert(&wrong_key, "unrelated fixture");
    let settings = Settings {
        sp_entity_id: SP.into(),
        acs_urls: vec![ACS.into()],
        acs_indices: BTreeMap::new(),
        idp_entity_id: None,
        idp_certificate_pem: idp_cert.clone(),
        sp_certificates_pem: vec![sp_cert.clone()],
        encryption_certificate_pem: None,
        name_id_format: NameIdFormat::Persistent,
        attributes: Vec::new(),
        slo_redirect_url: None,
        slo_post_url: None,
        idp_initiated: false,
        default_relay_state: None,
        assertion_ttl: 120,
    };
    fixture
        .core
        .create_client(
            &fixture.admin,
            NewClient {
                client_id: CLIENT.into(),
                name: CLIENT.into(),
                confidential: false,
                redirect_uris: Vec::new(),
                scopes: strings(&["openid", "saml", "profile"]),
                allowed_groups: BTreeSet::new(),
                require_mfa: false,
                service: false,
                settings: ProviderSettings {
                    saml: Some(settings),
                    ..Default::default()
                },
            },
        )
        .unwrap_or_else(|error| panic!("client: {error}"));
    let entity = format!("http://localhost:9000/saml/{CLIENT}/metadata");
    let metadata = fixture
        .core
        .saml_metadata(CLIENT)
        .unwrap_or_else(|error| panic!("metadata: {error}"));
    assert!(metadata.contains("WantAuthnRequestsSigned=\"true\""));
    let dir = fixture._dir.path();
    let sp_metadata_path = dir.join("sp-metadata.xml");
    let sp_key_path = dir.join("sp.key");
    let sp_cert_path = dir.join("sp.crt");
    let idp_metadata_path = dir.join("idp-metadata.xml");
    let wrong_metadata_path = dir.join("idp-wrong-metadata.xml");
    let state_path = dir.join("lasso-state");
    let response_path = dir.join("response.b64");
    let tamper_path = dir.join("tampered.b64");
    write_private(&sp_metadata_path, sp_metadata(&sp_cert).as_bytes());
    write_private(&sp_key_path, &sp_key.private_key_to_pem_pkcs8().unwrap());
    write_private(&sp_cert_path, sp_cert.as_bytes());
    let wrong_metadata = replace_certificates(&metadata, &wrong_cert);
    let idp_certificate = metadata
        .split_once("<ds:X509Certificate>")
        .expect("IdP metadata certificate")
        .1
        .split_once("</ds:X509Certificate>")
        .expect("IdP metadata certificate end")
        .0;
    assert!(metadata.matches(idp_certificate).count() >= 1);
    assert!(!wrong_metadata.contains(idp_certificate));
    write_private(&idp_metadata_path, metadata.as_bytes());
    write_private(&wrong_metadata_path, wrong_metadata.as_bytes());
    let prefix = |metadata: &Path| {
        vec![
            sp_metadata_path.display().to_string(),
            sp_key_path.display().to_string(),
            sp_cert_path.display().to_string(),
            metadata.display().to_string(),
            entity.clone(),
        ]
    };
    let mut request_args = vec!["request".to_string()];
    request_args.extend(prefix(&idp_metadata_path));
    request_args.push(ACS.to_string());
    request_args.push(state_path.display().to_string());
    let requested = fields(&run(&bin, &request_args));
    assert_eq!(requested["binding"], "redirect");
    assert_eq!(requested["relay"], "lasso-relay");
    let query: BTreeMap<String, String> =
        url::form_urlencoded::parse(requested["message"].as_bytes())
            .into_owned()
            .collect();
    assert_eq!(query["SigAlg"], RSA256);
    assert_eq!(query["RelayState"], "lasso-relay");
    assert!(query["SAMLRequest"].len() > 32);
    let started = fixture
        .core
        .saml_start(CLIENT, &requested["message"], false, None)
        .unwrap_or_else(|error| panic!("saml_start: {error}"));
    let (code, id, cookie) = waiting(started);
    let details = fixture
        .core
        .browser_details(&session, &code)
        .unwrap_or_else(|error| panic!("details: {error}"));
    let mut transaction = None;
    if details["reauthentication_required"] == true {
        let proof = details["transaction_id"]
            .as_str()
            .expect("transaction")
            .to_string();
        session = text(
            &fixture
                .core
                .login_for(
                    "lasso-user".into(),
                    PASSWORD.into(),
                    None,
                    Some(proof.clone()),
                )
                .unwrap(),
            "session_token",
        );
        transaction = Some(proof);
    }
    fixture
        .core
        .browser_decide(
            &session,
            BrowserDecision {
                code,
                approve: true,
                transaction_id: transaction,
                remember: false,
            },
        )
        .unwrap_or_else(|error| panic!("decide: {error}"));
    let (encoded, relay) = response_xml(
        fixture
            .core
            .saml_resume(&id, Some(&cookie))
            .unwrap_or_else(|error| panic!("resume: {error}")),
    );
    assert_eq!(relay, "lasso-relay");
    let xml = String::from_utf8(STANDARD.decode(&encoded).unwrap()).unwrap();
    assert!(xml.contains(&format!("Destination=\"{ACS}\"")));
    assert!(xml.contains(&format!(">{SP}<")));
    write_private(&response_path, encoded.as_bytes());
    let name_open = xml.find("<saml:NameID ").expect("name id");
    let name_at = name_open + xml[name_open..].find('>').expect("name id") + 1;
    let mut tampered = xml.as_bytes().to_vec();
    assert!(tampered[name_at].is_ascii_hexdigit());
    tampered[name_at] = if tampered[name_at] == b'a' {
        b'b'
    } else {
        b'a'
    };
    write_private(&tamper_path, STANDARD.encode(&tampered).as_bytes());
    let expired_path = dir.join("expired.b64");
    let expired_xml = resign(
        &expire_conditions(&without_signatures(&xml)),
        &idp_keys.active.pem,
        &idp_cert,
    );
    assert!(expired_xml.contains("<ds:Signature"));
    write_private(
        &expired_path,
        STANDARD.encode(expired_xml.as_bytes()).as_bytes(),
    );
    let recipient_path = dir.join("recipient.b64");
    let recipient_xml = resign(
        &retarget_recipient(&without_signatures(&xml)),
        &idp_keys.active.pem,
        &idp_cert,
    );
    assert!(recipient_xml.contains("<ds:Signature"));
    write_private(
        &recipient_path,
        STANDARD.encode(recipient_xml.as_bytes()).as_bytes(),
    );
    let accept = |metadata: &Path, response: &Path| {
        let mut args = vec!["accept".to_string()];
        args.extend(prefix(metadata));
        args.push(state_path.display().to_string());
        args.push(response.display().to_string());
        args.push(relay.clone());
        run(&bin, &args)
    };
    signature_reject(&accept(&idp_metadata_path, &tamper_path));
    signature_reject(&accept(&wrong_metadata_path, &response_path));
    lifetime_reject(&accept(&idp_metadata_path, &expired_path));
    recipient_reject(&accept(&idp_metadata_path, &recipient_path));
    let accepted = fields(&accept(&idp_metadata_path, &response_path));
    assert_eq!(accepted["signature"], "lasso");
    assert_eq!(accepted["audience"], "lasso");
    assert_eq!(accepted["lifetime"], "lasso");
    assert_eq!(accepted["recipient"], "helper");
    assert_eq!(accepted["format"], NameIdFormat::Persistent.uri());
    assert!(xml.contains(&format!(">{}<", accepted["name_id"])));
}
