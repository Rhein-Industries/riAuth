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

#[ignore = "requires the reviewed GNU Lasso SLO helper in RIAUTH_TEST_LASSO_SP"]
#[test]
fn lasso_idp_initiated_redirect_logout_revokes_only_bound_session_and_consumes_response_once() {
    use std::{
        io::Read,
        os::unix::{fs::OpenOptionsExt, process::ExitStatusExt},
        process::{Child, ExitStatus, Stdio},
        time::{Duration, Instant},
    };

    const CAP: u64 = 128 * 1024;
    const SLO: &str = "https://sp.example.test/logout";
    struct OwnedChild(Child);
    impl Drop for OwnedChild {
        fn drop(&mut self) {
            if !matches!(self.0.try_wait(), Ok(Some(_))) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
    }
    fn bounded_read(path: &Path) -> Vec<u8> {
        let mut bytes = Vec::new();
        fs::File::open(path)
            .expect("open private helper file")
            .take(CAP + 1)
            .read_to_end(&mut bytes)
            .expect("read private helper file");
        assert!(bytes.len() as u64 <= CAP, "helper file exceeds cap");
        bytes
    }
    fn private_text(path: &Path) -> String {
        let metadata = fs::symlink_metadata(path).expect("private file metadata");
        assert!(metadata.is_file(), "private file must be regular");
        assert!(metadata.permissions().mode() & 0o777 == 0o600);
        String::from_utf8(bounded_read(path)).unwrap_or_else(|_| panic!("private file UTF-8"))
    }
    fn run_bounded(
        bin: &Path,
        args: &[String],
        dir: &Path,
        sequence: usize,
        budget: Instant,
    ) -> (ExitStatus, BTreeMap<String, String>, String) {
        let stdout_path = dir.join(format!("helper-{sequence}.stdout"));
        let stderr_path = dir.join(format!("helper-{sequence}.stderr"));
        let output_file = |path: &Path| {
            fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(path)
                .expect("create exclusive private helper capture")
        };
        let deadline = (Instant::now() + Duration::from_secs(20)).min(budget);
        assert!(Instant::now() < deadline, "helper budget expired");
        let mut child = OwnedChild(
            Command::new(bin)
                .args(args)
                .env_clear()
                .current_dir(dir)
                .stdin(Stdio::null())
                .stdout(Stdio::from(output_file(&stdout_path)))
                .stderr(Stdio::from(output_file(&stderr_path)))
                .spawn()
                .expect("spawn owned Lasso helper"),
        );
        let status = loop {
            assert!(Instant::now() < deadline, "helper process deadline");
            for path in [&stdout_path, &stderr_path] {
                assert!(fs::metadata(path).expect("capture size").len() <= CAP);
            }
            if let Some(status) = child.0.try_wait().expect("poll owned helper") {
                break status;
            }
            // No timer/clock manipulation or sleep; the child also has a kernel alarm.
            std::thread::yield_now();
        };
        let stdout = String::from_utf8(bounded_read(&stdout_path))
            .unwrap_or_else(|_| panic!("helper output UTF-8"));
        let stderr = String::from_utf8(bounded_read(&stderr_path))
            .unwrap_or_else(|_| panic!("helper error UTF-8"));
        assert!(Instant::now() < deadline, "helper deadline after IO");
        let mut parsed = BTreeMap::new();
        if status.success() {
            for line in stdout.lines() {
                let (key, value) = line.split_once(": ").expect("fixed helper field framing");
                assert!(key.len() <= 24 && parsed.len() < 16);
                assert!(parsed.insert(key.to_owned(), value.to_owned()).is_none());
            }
        } else {
            assert!(
                stdout.is_empty(),
                "refused helper must not print private state"
            );
        }
        // Neither stdout/stderr nor protocol fields appear in panic diagnostics.
        (status, parsed, stderr)
    }

    let budget = Instant::now() + Duration::from_secs(60);
    let bin =
        PathBuf::from(std::env::var("RIAUTH_TEST_LASSO_SP").expect("select reviewed SLO helper"));
    let fixture = Fixture::new();
    let mut session = fixture.user("lasso-slo-user");
    let unrelated = fixture.user("lasso-unrelated");
    let idp_keys: riauth::crypto::Keys = fixture.core.store.get("meta", "keys").unwrap().unwrap();
    let idp_key = PKey::private_key_from_pem(idp_keys.active.pem.as_bytes()).unwrap();
    let idp_cert = cert(&idp_key, "riAuth SLO fixture");
    let sp_key = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();
    let sp_cert = cert(&sp_key, "Lasso SLO fixture");
    let settings = Settings {
        sp_entity_id: SP.into(),
        acs_urls: vec![ACS.into()],
        acs_indices: BTreeMap::new(),
        idp_entity_id: None,
        idp_certificate_pem: idp_cert,
        sp_certificates_pem: vec![sp_cert.clone()],
        encryption_certificate_pem: None,
        name_id_format: NameIdFormat::Persistent,
        attributes: Vec::new(),
        slo_redirect_url: Some(SLO.into()),
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
                name: "Lasso SLO".into(),
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
        .unwrap_or_else(|_| panic!("provision SLO client"));
    let entity = format!("http://localhost:9000/saml/{CLIENT}/metadata");
    let dir = fixture._dir.path();
    let sp_metadata_path = dir.join("slo-sp.xml");
    let sp_key_path = dir.join("slo-sp.key");
    let sp_cert_path = dir.join("slo-sp.crt");
    let idp_metadata_path = dir.join("slo-idp.xml");
    let login_path = dir.join("slo-login.dump");
    let response_path = dir.join("slo-response.b64");
    let identity_path = dir.join("slo-identity.dump");
    let session_path = dir.join("slo-session.dump");
    let request_path = dir.join("slo-request.query");
    let reply_path = dir.join("slo-reply.url");
    let retired_path = dir.join("slo-retired.dump");
    let sp_xml = sp_metadata(&sp_cert).replacen(
        "<NameIDFormat>",
        &format!(
            "<SingleLogoutService Binding=\"urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect\" Location=\"{SLO}\"/><NameIDFormat>"
        ),
        1,
    );
    write_private(&sp_metadata_path, sp_xml.as_bytes());
    write_private(&sp_key_path, &sp_key.private_key_to_pem_pkcs8().unwrap());
    write_private(&sp_cert_path, sp_cert.as_bytes());
    let metadata = fixture.core.saml_metadata(CLIENT).unwrap();
    write_private(&idp_metadata_path, metadata.as_bytes());
    for path in [
        &sp_metadata_path,
        &sp_key_path,
        &sp_cert_path,
        &idp_metadata_path,
    ] {
        let _ = private_text(path);
    }
    let prefix = |metadata: &Path| {
        vec![
            sp_metadata_path.display().to_string(),
            sp_key_path.display().to_string(),
            sp_cert_path.display().to_string(),
            metadata.display().to_string(),
            entity.clone(),
        ]
    };
    let mut sequence = 0;
    let mut execute = |args: &[String]| {
        let result = run_bounded(&bin, args, dir, sequence, budget);
        sequence += 1;
        result
    };
    let mut args = vec!["request".into()];
    args.extend(prefix(&idp_metadata_path));
    args.extend([ACS.into(), login_path.display().to_string()]);
    let (status, requested, _) = execute(&args);
    assert!(status.success(), "Lasso login request failed");
    assert!(requested["binding"] == "redirect");
    let (code, id, cookie) = waiting(
        fixture
            .core
            .saml_start(CLIENT, &requested["message"], false, None)
            .unwrap_or_else(|_| panic!("begin signed Lasso SSO")),
    );
    let details = fixture.core.browser_details(&session, &code).unwrap();
    let mut transaction = None;
    if details["reauthentication_required"] == true {
        let proof = details["transaction_id"].as_str().unwrap().to_owned();
        session = text(
            &fixture
                .core
                .login_for(
                    "lasso-slo-user".into(),
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
        .unwrap_or_else(|_| panic!("approve SLO fixture login"));
    let (encoded, relay) = response_xml(fixture.core.saml_resume(&id, Some(&cookie)).unwrap());
    let xml = String::from_utf8(STANDARD.decode(&encoded).unwrap())
        .unwrap_or_else(|_| panic!("issued response UTF-8"));
    let doc = roxmltree::Document::parse(&xml).unwrap();
    let issued_name = doc
        .descendants()
        .find(|node| node.has_tag_name(("urn:oasis:names:tc:SAML:2.0:assertion", "NameID")))
        .unwrap()
        .text()
        .unwrap()
        .to_owned();
    let issued_index = doc
        .descendants()
        .find(|node| node.has_tag_name(("urn:oasis:names:tc:SAML:2.0:assertion", "AuthnStatement")))
        .unwrap()
        .attribute("SessionIndex")
        .unwrap()
        .to_owned();
    assert!(!issued_name.is_empty() && !issued_index.is_empty());
    write_private(&response_path, encoded.as_bytes());
    let mut args = vec!["accept-state".into()];
    args.extend(prefix(&idp_metadata_path));
    args.extend([
        login_path.display().to_string(),
        response_path.display().to_string(),
        relay,
        identity_path.display().to_string(),
        session_path.display().to_string(),
    ]);
    let (status, accepted, _) = execute(&args);
    assert!(status.success(), "Lasso accepted-state persistence failed");
    assert!(accepted["accepted"] == "true" && accepted["persisted"] == "true");
    let identity_before = private_text(&identity_path);
    let session_before = private_text(&session_path);
    let state_args = |saved: &Path| {
        let mut args = vec!["session-state".into()];
        args.extend(prefix(&idp_metadata_path));
        args.extend([
            identity_path.display().to_string(),
            saved.display().to_string(),
        ]);
        args
    };
    let (status, state, _) = execute(&state_args(&session_path));
    assert!(status.success(), "Lasso saved-state inspection failed");
    assert!(state["assertion"] == "present");
    assert!(state["names"] == "1" && state["indices"] == "1" && state["empty"] == "false");

    let count_confirmations = || {
        fixture
            .core
            .store
            .list::<serde_json::Value>("audit")
            .unwrap()
            .iter()
            .filter(|(_, row)| {
                row["action"] == "saml.logout.confirmation" && row["target"] == CLIENT
            })
            .count()
    };
    assert!(count_confirmations() == 0);
    let revoked = fixture.core.logout(&session).unwrap();
    assert!(
        fixture.core.me(&session).is_err(),
        "local revocation precedes delivery"
    );
    assert!(fixture.core.me(&unrelated).is_ok());
    let ticket = revoked["saml_logout_url"]
        .as_str()
        .unwrap()
        .rsplit('/')
        .next()
        .unwrap()
        .to_owned();
    let Reply::Redirect(request_url) = fixture.core.saml_logout_next(&ticket).unwrap() else {
        panic!("expected Redirect SLO request");
    };
    let Reply::Redirect(retry_url) = fixture.core.saml_logout_next(&ticket).unwrap() else {
        panic!("expected identical pending request");
    };
    assert!(retry_url == request_url, "pending request retry is stable");
    let request_url = url::Url::parse(&request_url).unwrap();
    assert!(request_url.as_str().split('?').next() == Some(SLO));
    let request_query = request_url.query().unwrap();
    let request_fields: BTreeMap<_, _> = request_url.query_pairs().into_owned().collect();
    assert!(request_fields["SigAlg"] == RSA256 && request_fields["RelayState"] == ticket);
    let compressed = STANDARD.decode(&request_fields["SAMLRequest"]).unwrap();
    let mut request_xml = String::new();
    flate2::read::DeflateDecoder::new(compressed.as_slice())
        .take(CAP + 1)
        .read_to_string(&mut request_xml)
        .unwrap();
    assert!(request_xml.len() as u64 <= CAP);
    let request_doc = roxmltree::Document::parse(&request_xml).unwrap();
    let name = request_doc
        .descendants()
        .find(|node| node.has_tag_name(("urn:oasis:names:tc:SAML:2.0:assertion", "NameID")))
        .unwrap();
    let indices: Vec<_> = request_doc
        .descendants()
        .filter(|node| node.has_tag_name(("urn:oasis:names:tc:SAML:2.0:protocol", "SessionIndex")))
        .collect();
    assert!(
        name.text() == Some(issued_name.as_str()),
        "exact accepted NameID"
    );
    assert!(indices.len() == 1 && indices[0].text() == Some(issued_index.as_str()));
    write_private(&request_path, request_query.as_bytes());
    let logout_args = |metadata: &Path, request: &Path, reply: &Path, retired: &Path| {
        let mut args = vec!["logout".into()];
        args.extend(prefix(metadata));
        args.extend([
            identity_path.display().to_string(),
            session_path.display().to_string(),
            request.display().to_string(),
            reply.display().to_string(),
            retired.display().to_string(),
        ]);
        args
    };

    // Actual Lasso signature refusals leave saved accepted state and outputs untouched.
    let wrong_key = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();
    let wrong_path = dir.join("slo-untrusted-idp.xml");
    write_private(
        &wrong_path,
        replace_certificates(&metadata, &cert(&wrong_key, "unrelated")).as_bytes(),
    );
    let refused_reply = dir.join("slo-refused.url");
    let refused_state = dir.join("slo-refused.dump");
    let (status, _, stderr) = execute(&logout_args(
        &wrong_path,
        &request_path,
        &refused_reply,
        &refused_state,
    ));
    fn refusal_projection(stderr: &str) -> Option<(&'static str, i32)> {
        const STAGES: &[&str] = &[
            "lasso_init",
            "lasso_server_new",
            "lasso_server_add_provider",
            "lasso_server_get_provider",
            "lasso_logout_new",
            "lasso_profile_set_identity_from_dump",
            "lasso_profile_set_session_from_dump",
            "lasso_logout_process_request_msg",
            "lasso_profile_get_signature_status",
            "lasso_logout_validate_request",
            "lasso_logout_build_response_msg",
        ];
        if stderr.len() as u64 > CAP {
            return None;
        }
        let mut projected = None;
        for raw in stderr.split_inclusive('\n') {
            if !raw.starts_with("lasso ") {
                continue;
            }
            if projected.is_some() {
                return None;
            }
            let line = raw.strip_suffix('\n')?.strip_prefix("lasso ")?;
            let (name, detail) = line.split_once(": ")?;
            let stage = STAGES.iter().copied().find(|stage| *stage == name)?;
            let (text, decimal) = detail.strip_suffix(')')?.rsplit_once(" (")?;
            let code = decimal.parse::<i32>().ok()?;
            if text.is_empty()
                || text.bytes().any(|byte| byte.is_ascii_control())
                || code.to_string() != decimal
            {
                return None;
            }
            projected = Some((stage, code));
        }
        projected
    }
    assert!(
        status.code() == Some(1)
            && refusal_projection(&stderr) == Some(("lasso_logout_process_request_msg", 102)),
        "pinned signature refusal: exit={:?} signal={:?} projection={:?}",
        status.code(),
        status.signal(),
        refusal_projection(&stderr)
    );
    assert!(!refused_reply.exists() && !refused_state.exists());
    assert!(private_text(&identity_path) == identity_before);
    assert!(private_text(&session_path) == session_before);

    let (status, processed, _) = execute(&logout_args(
        &idp_metadata_path,
        &request_path,
        &reply_path,
        &retired_path,
    ));
    assert!(status.success(), "Lasso logout receiver failed");
    assert!(processed["signature"] == "lasso" && processed["matched_indices"] == "1");
    assert!(processed["assertion_after"] == "absent");
    assert!(processed["indices_after"] == "0" && processed["empty_after"] == "true");
    // Restore the actual serialized empty Lasso object in a separate process.
    let _ = private_text(&retired_path);
    let (status, saved, _) = execute(&state_args(&retired_path));
    assert!(status.success(), "Lasso retired-state reload failed");
    assert!(saved["assertion"] == "absent");
    assert!(saved["names"] == "0" && saved["indices"] == "0" && saved["empty"] == "true");
    let retired_before_retry = private_text(&retired_path);
    let mut spent_args = logout_args(
        &idp_metadata_path,
        &request_path,
        &refused_reply,
        &refused_state,
    );
    // argv[8] in C (Rust args[7]) is the saved session, not the request query.
    spent_args[7] = retired_path.display().to_string();
    let (status, _, stderr) = execute(&spent_args);
    assert!(status.code() == Some(1) && stderr == "lasso slo: issued session binding\n");
    assert!(!refused_reply.exists() && !refused_state.exists());
    assert!(private_text(&retired_path) == retired_before_retry);
    assert!(private_text(&identity_path) == identity_before);
    assert!(private_text(&session_path) == session_before);
    assert!(
        count_confirmations() == 0,
        "peer response not yet delivered"
    );

    let reply_url = url::Url::parse(&private_text(&reply_path)).unwrap();
    assert!(
        reply_url.as_str().split('?').next()
            == Some(format!("http://localhost:9000/saml/{CLIENT}/sso").as_str())
    );
    let fields: BTreeMap<_, _> = reply_url.query_pairs().into_owned().collect();
    assert!(fields["SigAlg"] == RSA256 && fields["RelayState"] == ticket);
    assert!(fields.contains_key("Signature") && fields.contains_key("SAMLResponse"));
    let query = reply_url.query().unwrap();
    let Reply::Redirect(continuation) =
        fixture.core.saml_start(CLIENT, query, false, None).unwrap()
    else {
        panic!("expected accepted logout continuation");
    };
    assert!(continuation == revoked["saml_logout_url"].as_str().unwrap());
    let status = fixture.core.saml_logout_status(&ticket).unwrap();
    assert!(status["status"] == "complete" && status["confirmed"] == 1);
    assert!(status["remaining"] == 0 && status["failed"] == 0);
    assert!(count_confirmations() == 1);
    let before_retry = fixture.core.store.read(|tx| tx.snapshot()).unwrap();
    assert!(
        fixture.core.saml_start(CLIENT, query, false, None).is_err(),
        "consumed response refuses retry"
    );
    assert!(
        fixture.core.store.read(|tx| tx.snapshot()).unwrap() == before_retry,
        "retry snapshot unchanged"
    );
    assert!(count_confirmations() == 1);
    assert!(fixture.core.me(&session).is_err() && fixture.core.me(&unrelated).is_ok());
    assert!(Instant::now() < budget, "fixture budget after final checks");
}
