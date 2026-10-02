//! Independent native GNU Lasso IdP -> riAuth source lifecycle, without a listener.
#![cfg(all(feature = "platform", target_os = "macos"))]

use base64::{Engine, engine::general_purpose::STANDARD};
use openssl::{
    asn1::Asn1Time,
    hash::MessageDigest,
    pkey::{PKey, Private},
    rsa::Rsa,
    x509::{X509, X509NameBuilder},
};
use riauth::{
    config::Config,
    core::Core,
    crypto::digest,
    error::Result,
    jose::ClientAuthMethod,
    keyring::KeyInput,
    model::NewUser,
    saml::NameIdFormat,
    source::{Finish, Source, SourceInput, Start, saml::Settings},
};
use roxmltree::{Document, Node};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::{
        fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
        process::ExitStatusExt,
    },
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    time::{Duration, Instant},
};

// Selected macOS SDK sys/fcntl.h: O_NOFOLLOW; this native target makes no
// portability claim for other hosts and introduces no dependency.
const NOFOLLOW: i32 = 0x00000100;
const CAP: u64 = 128 * 1024;
const SOURCE: &str = "lasso-idp-source";
const IDP: &str = "urn:example:lasso:local-idp";
const SP: &str = "urn:example:riauth:lasso-source";
const SSO: &str = "https://idp.example.test/sso";
const P: &str = "urn:oasis:names:tc:SAML:2.0:protocol";
const A: &str = "urn:oasis:names:tc:SAML:2.0:assertion";
const DS: &str = "http://www.w3.org/2000/09/xmldsig#";
const RSA256: &str = "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256";
const POST: &str = "urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST";
const REDIRECT: &str = "urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect";
const PASSWORD: &str = "independent-native-peer-fixture-only-password";
type Snapshot = BTreeMap<String, Value>;

// Deliberately omit error values: private protocol/state never enters diagnostics.
fn must<T, E>(result: std::result::Result<T, E>, stage: &'static str) -> T {
    match result {
        Ok(value) => value,
        Err(_) => panic!("{stage}"),
    }
}
fn text(value: &Value, field: &str) -> String {
    value[field].as_str().expect("public result field").into()
}
fn live(budget: Instant) {
    assert!(Instant::now() < budget, "whole-test 60-second deadline");
}
fn snapshot(core: &Core) -> Snapshot {
    must(core.store.read(|tx| tx.snapshot()), "read-only snapshot")
}
fn same(core: &Core, before: &Snapshot) {
    assert!(&snapshot(core) == before, "full snapshot changed");
}
fn audit_count(snapshot: &Snapshot, action: &str) -> usize {
    snapshot
        .iter()
        .filter(|(key, value)| key.starts_with("audit/") && value["action"] == action)
        .count()
}
fn count(snapshot: &Snapshot, bucket: &str) -> usize {
    let prefix = format!("{bucket}/");
    snapshot
        .keys()
        .filter(|key| key.starts_with(&prefix))
        .count()
}
fn refusal<T>(result: Result<T>, status: u16, message: Option<&str>) {
    let error = match result {
        Ok(_) => panic!("expected public refusal"),
        Err(error) => error,
    };
    assert!(error.status.as_u16() == status, "public refusal status");
    assert!(
        error.code
            == match status {
                400 => "invalid_request",
                401 => "invalid_token",
                403 => "access_denied",
                _ => panic!("unlisted public refusal status"),
            },
        "public refusal code"
    );
    if let Some(expected) = message {
        assert!(error.message == expected, "public refusal description");
    }
}

// Compare *all* domains/indexes/counters/receipts/revisions outside the exact
// changed pending record and one new denial audit, rather than broad exclusions.
fn sealed_denial(core: &Core, before: &Snapshot, relay: &str) {
    let after = snapshot(core);
    let key = format!("source_logins/{}", digest(relay));
    let mut expected = before[&key].clone();
    assert!(expected["claimed"] == false && expected["failed"] == false);
    assert!(expected["result"].is_null());
    expected["claimed"] = Value::Bool(true);
    expected["failed"] = Value::Bool(true);
    assert!(
        after.get(&key) == Some(&expected),
        "exact terminal control record"
    );
    let additions: Vec<_> = after
        .iter()
        .filter(|(name, _)| !before.contains_key(*name))
        .collect();
    assert!(additions.len() == 1, "one denial record only");
    let (audit_key, event) = additions[0];
    let audit_key = audit_key.clone();
    assert!(audit_key.starts_with("audit/"));
    assert!(event["actor"] == "upstream" && event["action"] == "source.login_failed");
    assert!(event["target"] == SOURCE);
    assert!(
        audit_count(&after, "source.login_failed")
            == audit_count(before, "source.login_failed") + 1
    );
    let mut protected_before = before.clone();
    let mut protected_after = after;
    protected_before.remove(&key);
    protected_after.remove(&key);
    protected_after.remove(&audit_key);
    assert!(
        protected_after == protected_before,
        "exact protected-domain equality"
    );
}

fn new_user(username: &str, admin: bool) -> NewUser {
    NewUser {
        username: username.into(),
        password: PASSWORD.into(),
        email: None,
        display_name: username.into(),
        admin,
    }
}
fn login(core: &Core, username: &str) -> String {
    text(
        &must(
            core.login(username.into(), PASSWORD.into(), None),
            "local login",
        ),
        "session_token",
    )
}
fn certificate(key: &PKey<Private>, name: &str) -> (String, String) {
    let mut subject = must(X509NameBuilder::new(), "certificate name");
    must(subject.append_entry_by_text("CN", name), "certificate CN");
    let subject = subject.build();
    let mut cert = must(X509::builder(), "certificate builder");
    must(cert.set_version(2), "certificate version");
    let serial = must(openssl::bn::BigNum::from_u32(1), "serial");
    let serial = must(serial.to_asn1_integer(), "serial ASN1");
    must(cert.set_serial_number(&serial), "certificate serial");
    must(cert.set_subject_name(&subject), "certificate subject");
    must(cert.set_issuer_name(&subject), "certificate issuer");
    must(cert.set_pubkey(key), "certificate public key");
    must(
        cert.set_not_before(&must(Asn1Time::days_from_now(0), "not before")),
        "certificate validity start",
    );
    must(
        cert.set_not_after(&must(Asn1Time::days_from_now(1), "not after")),
        "certificate validity end",
    );
    must(
        cert.sign(key, MessageDigest::sha256()),
        "certificate signature",
    );
    let cert = cert.build();
    (
        must(
            String::from_utf8(must(cert.to_pem(), "certificate PEM")),
            "certificate UTF8",
        ),
        STANDARD.encode(must(cert.to_der(), "certificate DER")),
    )
}
fn idp_metadata(cert: &str) -> String {
    // Fixture configuration only. All success protocol messages are built by Lasso.
    format!(
        r#"<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata" xmlns:ds="{DS}" entityID="{IDP}"><md:IDPSSODescriptor protocolSupportEnumeration="{P}" WantAuthnRequestsSigned="true"><md:KeyDescriptor use="signing"><ds:KeyInfo><ds:X509Data><ds:X509Certificate>{cert}</ds:X509Certificate></ds:X509Data></ds:KeyInfo></md:KeyDescriptor><md:NameIDFormat>{}</md:NameIDFormat><md:SingleSignOnService Binding="{REDIRECT}" Location="{SSO}"/></md:IDPSSODescriptor></md:EntityDescriptor>"#,
        NameIdFormat::Persistent.uri()
    )
}

fn private_file(path: &Path, uid: u32, nonempty: bool) -> File {
    let mut options = OpenOptions::new();
    options.read(true).custom_flags(NOFOLLOW);
    let file = must(options.open(path), "private input open");
    let metadata = must(file.metadata(), "private input metadata");
    assert!(metadata.is_file() && metadata.uid() == uid && metadata.nlink() == 1);
    assert!(metadata.mode() & 0o777 == 0o600 && metadata.len() <= CAP);
    assert!(!nonempty || metadata.len() > 0);
    file
}
fn private_read(path: &Path, uid: u32, nonempty: bool) -> Vec<u8> {
    let file = private_file(path, uid, nonempty);
    let mut bytes = Vec::new();
    must(
        file.take(CAP + 1).read_to_end(&mut bytes),
        "private input read",
    );
    assert!(bytes.len() as u64 <= CAP && (!nonempty || !bytes.is_empty()));
    bytes
}
fn private_write(path: &Path, bytes: &[u8]) {
    assert!(!bytes.is_empty() && bytes.len() as u64 <= CAP);
    let mut file = must(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(NOFOLLOW)
            .open(path),
        "exclusive private write",
    );
    must(file.write_all(bytes), "private write");
}
fn capture(path: &Path) -> File {
    must(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(NOFOLLOW)
            .open(path),
        "private capture",
    )
}

struct OwnedChild(Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
struct NativeResult {
    status: ExitStatus,
    stdout: Vec<u8>,
    diagnostic: Option<(&'static str, i32)>,
}
fn projection(bytes: &[u8]) -> Option<(&'static str, i32)> {
    const STAGES: [&str; 8] = [
        "lasso_init",
        "lasso_server_add_provider_from_buffer",
        "lasso_profile_set_identity_from_dump",
        "lasso_login_process_authn_request_msg",
        "lasso_profile_get_signature_status",
        "lasso_login_validate_request_msg",
        "lasso_login_build_assertion",
        "lasso_login_build_authn_response_msg",
    ];
    let stderr = std::str::from_utf8(bytes).ok()?;
    let mut found = None;
    for line in stderr.lines() {
        if !line.starts_with("lasso idp ") {
            continue;
        }
        let mut matched = false;
        for stage in STAGES {
            let prefix = format!("lasso idp {stage} (");
            if let Some(code) = line
                .strip_prefix(&prefix)
                .and_then(|rest| rest.strip_suffix(')'))
            {
                let number = code.parse::<i32>().ok()?;
                if number == 0 || number.to_string() != code || found.is_some() {
                    return None;
                }
                found = Some((stage, number));
                matched = true;
            }
        }
        if !matched {
            return None;
        }
    }
    found
}

struct NativePeer {
    bin: PathBuf,
    dir: PathBuf,
    uid: u32,
    children: usize,
    budget: Instant,
}
struct NativeOutput {
    identity: PathBuf,
    session: PathBuf,
    post: PathBuf,
}
impl NativePeer {
    fn run(
        &mut self,
        inputs: &[PathBuf; 5],
        identity: Option<&Path>,
    ) -> (NativeResult, NativeOutput) {
        live(self.budget);
        self.children += 1;
        assert!(self.children <= 5, "five native children maximum");
        let n = self.children;
        let output = NativeOutput {
            identity: self.dir.join(format!("identity-{n}.xml")),
            session: self.dir.join(format!("session-{n}.xml")),
            post: self.dir.join(format!("post-{n}.txt")),
        };
        let mut protected = inputs.to_vec();
        if let Some(identity) = identity {
            protected.push(identity.into());
        }
        let before: Vec<_> = protected
            .iter()
            .map(|p| private_read(p, self.uid, true))
            .collect();
        let mut names = BTreeSet::new();
        for path in protected
            .iter()
            .chain([&output.identity, &output.session, &output.post])
        {
            assert!(
                path.parent() == Some(self.dir.as_path()),
                "own private siblings"
            );
            assert!(names.insert(path), "distinct private paths");
        }
        for path in [&output.identity, &output.session, &output.post] {
            assert!(
                fs::symlink_metadata(path).is_err(),
                "exclusive output absent"
            );
        }
        let stdout_path = self.dir.join(format!("native-{n}.stdout"));
        let stderr_path = self.dir.join(format!("native-{n}.stderr"));
        let deadline = self.budget.min(Instant::now() + Duration::from_secs(20));
        // Regular capped files avoid an inherited-pipe reader blocking after exit.
        // Child remains in the outer Cargo process group; Drop kills/reaps it.
        let mut command = Command::new(&self.bin);
        command
            .env_clear()
            .stdin(Stdio::null())
            .stdout(capture(&stdout_path))
            .stderr(capture(&stderr_path))
            .arg("idp-login")
            .args(&inputs[..4])
            .arg(SP)
            .arg(&inputs[4]);
        if let Some(identity) = identity {
            command.arg(identity);
        } else {
            command.arg("-");
        }
        command
            .arg(&output.identity)
            .arg(&output.session)
            .arg(&output.post);
        let mut child = OwnedChild(must(command.spawn(), "native helper spawn"));
        let status = loop {
            assert!(Instant::now() < deadline, "native helper deadline");
            for path in [&stdout_path, &stderr_path] {
                assert!(
                    must(fs::metadata(path), "capture metadata").len() <= CAP,
                    "native capture cap"
                );
            }
            if let Some(status) = must(child.0.try_wait(), "native child status") {
                break status;
            }
            std::thread::yield_now();
        };
        let stdout = private_read(&stdout_path, self.uid, false);
        let stderr = private_read(&stderr_path, self.uid, false);
        for (path, original) in protected.iter().zip(before.iter()) {
            assert!(
                &private_read(path, self.uid, true) == original,
                "native input changed"
            );
        }
        assert!(
            Instant::now() < deadline,
            "native deadline after status and IO"
        );
        (
            NativeResult {
                status,
                stdout,
                diagnostic: projection(&stderr),
            },
            output,
        )
    }
    fn accepted(&mut self, inputs: &[PathBuf; 5], identity: Option<&Path>) -> NativeOutput {
        let (result, output) = self.run(inputs, identity);
        assert!(
            result.status.success(),
            "native exit={:?} signal={:?} stage_code={:?}",
            result.status.code(),
            result.status.signal(),
            result.diagnostic
        );
        assert!(result.stdout == b"binding: post\nrequest_signature: lasso\nassertion_built: lasso\nresponse_built: lasso\npersisted: true\n", "fixed native status");
        for path in [&output.identity, &output.session, &output.post] {
            let bytes = private_read(path, self.uid, true);
            assert!(!bytes.contains(&0), "native private framing");
        }
        live(self.budget);
        output
    }
}

struct Reservation {
    query: PathBuf,
    relay: String,
    credential: String,
    request_id: String,
    started_at: u64,
}
fn start(core: &Core, peer: &NativePeer, label: &str, token: Option<&str>) -> Reservation {
    live(peer.budget);
    let body = must(
        core.source_start(
            SOURCE,
            Start {
                link: token.is_some(),
                authentication_transaction: None,
            },
            token,
        ),
        "public source start",
    );
    let url = must(
        url::Url::parse(&text(&body, "authorization_url")),
        "source Redirect URL",
    );
    assert!(url.as_str().starts_with(&format!("{SSO}?")));
    let query = url.query().expect("source query");
    assert!(query.len() <= 65536 && !query.contains(['\r', '\n', ';']));
    let pairs: BTreeMap<_, _> = url
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    assert!(pairs.len() == 4 && pairs.get("SigAlg").is_some_and(|v| v == RSA256));
    let relay = pairs.get("RelayState").expect("original relay").clone();
    assert!(relay.len() == 43);
    let pending = &snapshot(core)[&format!("source_logins/{}", digest(&relay))];
    let request_id = format!("_{}", text(pending, "nonce"));
    let started_at = pending["started_at"]
        .as_u64()
        .expect("normal reserved time");
    let path = peer.dir.join(format!("query-{label}.txt"));
    private_write(&path, query.as_bytes());
    live(peer.budget);
    Reservation {
        query: path,
        relay,
        credential: text(&body["credential"], "token"),
        request_id,
        started_at,
    }
}
fn child<'a, 'input>(node: Node<'a, 'input>, ns: &str, name: &str) -> Node<'a, 'input> {
    let nodes: Vec<_> = node
        .children()
        .filter(|n| n.has_tag_name((ns, name)))
        .collect();
    assert!(nodes.len() == 1, "one required native XML child");
    nodes[0]
}
fn time(value: Option<&str>) -> u64 {
    let at = must(
        time::OffsetDateTime::parse(
            value.expect("native timestamp"),
            &time::format_description::well_known::Rfc3339,
        ),
        "native timestamp format",
    )
    .unix_timestamp();
    must(u64::try_from(at), "nonnegative native time")
}
struct NativePost {
    response: String,
    subject: String,
    index: String,
}
fn post(
    core: &Core,
    peer: &NativePeer,
    output: &NativeOutput,
    request: &Reservation,
    cert: &str,
) -> NativePost {
    live(peer.budget);
    let file = must(
        String::from_utf8(private_read(&output.post, peer.uid, true)),
        "POST bundle UTF8",
    );
    assert!(file.ends_with('\n'));
    let lines: Vec<_> = file.lines().collect();
    assert!(lines.len() == 3);
    let target = lines[0]
        .strip_prefix("target: ")
        .expect("private target framing");
    let relay = lines[1]
        .strip_prefix("relay: ")
        .expect("private relay framing");
    let response = lines[2]
        .strip_prefix("response: ")
        .expect("private response framing");
    let acs = core.saml_source_callback_url(SOURCE);
    assert!(
        target == acs && target.len() <= 1024 && relay == request.relay && response.len() <= 65536
    );
    let bytes = must(STANDARD.decode(response), "native response base64");
    assert!(!bytes.is_empty() && bytes.len() <= 48 * 1024);
    let xml = must(String::from_utf8(bytes), "native response UTF8");
    let doc = must(Document::parse(&xml), "native response XML");
    let root = doc.root_element();
    assert!(root.has_tag_name((P, "Response")) && root.attribute("Version") == Some("2.0"));
    assert!(
        root.attribute("Destination") == Some(acs.as_str())
            && root.attribute("InResponseTo") == Some(request.request_id.as_str())
    );
    assert!(child(root, A, "Issuer").text() == Some(IDP));
    assert!(
        child(child(root, P, "Status"), P, "StatusCode").attribute("Value")
            == Some("urn:oasis:names:tc:SAML:2.0:status:Success")
    );
    let assertions: Vec<_> = doc
        .descendants()
        .filter(|n| n.has_tag_name((A, "Assertion")))
        .collect();
    assert!(assertions.len() == 1 && assertions[0].parent() == Some(root));
    assert!(
        !doc.descendants().any(
            |n| n.has_tag_name((A, "EncryptedAssertion")) || n.has_tag_name((A, "EncryptedID"))
        )
    );
    let assertion = assertions[0];
    assert!(
        assertion.attribute("Version") == Some("2.0")
            && child(assertion, A, "Issuer").text() == Some(IDP)
    );
    let signatures: Vec<_> = doc
        .descendants()
        .filter(|n| n.has_tag_name((DS, "Signature")))
        .collect();
    assert!(
        signatures.len() == 2
            && signatures[0].parent() == Some(root)
            && signatures[1].parent() == Some(assertion)
    );
    let mut ids = BTreeSet::new();
    for node in doc.descendants().filter(Node::is_element) {
        if let Some(id) = node.attribute("ID") {
            assert!(!id.is_empty() && ids.insert(id), "unique native XML ID");
        }
    }
    for (signature, owner) in signatures.iter().zip([root, assertion]) {
        let info = child(*signature, DS, "SignedInfo");
        assert!(
            child(info, DS, "CanonicalizationMethod").attribute("Algorithm")
                == Some("http://www.w3.org/2001/10/xml-exc-c14n#")
        );
        assert!(child(info, DS, "SignatureMethod").attribute("Algorithm") == Some(RSA256));
        let reference = child(info, DS, "Reference");
        let expected = format!(
            "#{}",
            owner.attribute("ID").expect("native signed owner ID")
        );
        assert!(reference.attribute("URI") == Some(expected.as_str()));
        assert!(
            child(reference, DS, "DigestMethod").attribute("Algorithm")
                == Some("http://www.w3.org/2001/04/xmlenc#sha256")
        );
        let transforms: Vec<_> = child(reference, DS, "Transforms")
            .children()
            .filter(Node::is_element)
            .collect();
        assert!(
            transforms.len() == 2 && transforms.iter().all(|n| n.has_tag_name((DS, "Transform")))
        );
        assert!(
            transforms[0].attribute("Algorithm")
                == Some("http://www.w3.org/2000/09/xmldsig#enveloped-signature")
                && transforms[1].attribute("Algorithm")
                    == Some("http://www.w3.org/2001/10/xml-exc-c14n#")
        );
        let key = child(*signature, DS, "KeyInfo");
        assert!(key.children().filter(Node::is_element).count() == 1);
        let data = child(key, DS, "X509Data");
        assert!(data.children().filter(Node::is_element).count() == 1);
        let encoded = child(data, DS, "X509Certificate")
            .text()
            .expect("native certificate");
        let encoded: String = encoded
            .chars()
            .filter(|c| !c.is_ascii_whitespace())
            .collect();
        assert!(encoded == cert, "native configured certificate");
        assert!(
            child(*signature, DS, "SignatureValue")
                .text()
                .is_some_and(|v| !v.trim().is_empty())
        );
    }
    let subject = child(assertion, A, "Subject");
    let name = child(subject, A, "NameID");
    assert!(name.attribute("Format") == Some(NameIdFormat::Persistent.uri()));
    assert!(
        name.attribute("NameQualifier") == Some(IDP)
            && name.attribute("SPNameQualifier") == Some(SP)
    );
    let subject_name = name.text().expect("native persistent subject");
    assert!(
        !subject_name.is_empty()
            && subject_name.len() <= 255
            && !subject_name.chars().any(char::is_control)
    );
    let confirmation = child(subject, A, "SubjectConfirmation");
    assert!(confirmation.attribute("Method") == Some("urn:oasis:names:tc:SAML:2.0:cm:bearer"));
    let data = child(confirmation, A, "SubjectConfirmationData");
    assert!(
        data.attribute("Recipient") == Some(acs.as_str())
            && data.attribute("InResponseTo") == Some(request.request_id.as_str())
    );
    let conditions = child(assertion, A, "Conditions");
    assert!(child(child(conditions, A, "AudienceRestriction"), A, "Audience").text() == Some(SP));
    let auth = child(assertion, A, "AuthnStatement");
    let index = auth
        .attribute("SessionIndex")
        .expect("native application SessionIndex");
    assert!(Some(index) == assertion.attribute("ID") && !index.is_empty() && index.len() <= 256);
    assert!(
        child(child(auth, A, "AuthnContext"), A, "AuthnContextClassRef").text()
            == Some("urn:oasis:names:tc:SAML:2.0:ac:classes:PasswordProtectedTransport")
    );
    let now = riauth::crypto::now();
    let auth_time = time(auth.attribute("AuthnInstant"));
    assert!(auth_time >= request.started_at && auth_time <= now + 30);
    assert!(time(root.attribute("IssueInstant")) >= request.started_at.saturating_sub(1));
    assert!(time(assertion.attribute("IssueInstant")) >= request.started_at.saturating_sub(1));
    assert!(time(conditions.attribute("NotBefore")) == auth_time);
    for expiry in [
        data.attribute("NotOnOrAfter"),
        conditions.attribute("NotOnOrAfter"),
        auth.attribute("SessionNotOnOrAfter"),
    ] {
        assert!(time(expiry) == auth_time + 300 && time(expiry) > now);
    }
    // Structure checks do not substitute for Core's independent verification.
    // Pass the exact native base64/relay without rewriting or manual re-signing.
    live(peer.budget);
    NativePost {
        response: response.into(),
        subject: subject_name.into(),
        index: index.into(),
    }
}
fn callback(core: &Core, post: &NativePost, request: &Reservation) -> Result<Value> {
    core.saml_source_callback(
        SOURCE,
        vec![
            ("SAMLResponse".into(), post.response.clone()),
            ("RelayState".into(), request.relay.clone()),
        ],
    )
}
fn finish(core: &Core, request: &Reservation, approve: bool) -> Result<Value> {
    core.source_finish(Finish {
        credential: request.credential.clone(),
        approve,
        otp: None,
    })
}
fn complete_link(
    core: &Core,
    peer: &NativePeer,
    request: &Reservation,
    native: &NativePost,
    alice: &str,
) -> String {
    live(peer.budget);
    let before = snapshot(core);
    let accepted = must(
        callback(core, native, request),
        "public native source callback",
    );
    assert!(accepted["completed"] == true);
    let after = snapshot(core);
    assert!(
        audit_count(&after, "source.authenticated")
            == audit_count(&before, "source.authenticated") + 1
    );
    assert!(count(&after, "saml_source_replays") == count(&before, "saml_source_replays") + 1);
    refusal(
        callback(core, native, request),
        400,
        Some("SAML source request expired or already used"),
    );
    same(core, &after);
    let reviewed = must(finish(core, request, false), "public source review");
    assert!(
        reviewed["status"] == "review" && reviewed["linking"] == true && reviewed["mfa"] == false
    );
    assert!(reviewed["local_user"]["id"] == alice && reviewed["subject"] == native.subject);
    same(core, &after);
    let complete = must(finish(core, request, true), "explicit source link approval");
    assert!(complete["status"] == "complete" && complete["user"]["id"] == alice);
    let linked = snapshot(core);
    assert!(count(&linked, "source_links") == count(&after, "source_links") + 1);
    assert!(count(&linked, "sessions") == count(&after, "sessions") + 1);
    assert!(count(&linked, "saml_source_sessions") == count(&after, "saml_source_sessions") + 1);
    assert!(audit_count(&linked, "source.link") == audit_count(&after, "source.link") + 1);
    refusal(finish(core, request, true), 401, None);
    same(core, &linked);
    let token = text(&complete, "session_token");
    let identity = must(core.me(&token), "issued source identity");
    assert!(identity["user"]["id"] == alice && identity["mfa"] == false);
    let sid = text(&identity, "session_id");
    let session = &linked[&format!("saml_source_sessions/{sid}")];
    assert!(session["subject"] == native.subject && session["index"] == native.index);
    live(peer.budget);
    token
}
fn locals(core: &Core, alice: &str, unrelated: &str, alice_id: &str, unrelated_id: &str) {
    assert!(must(core.me(alice), "local Alice survives")["user"]["id"] == alice_id);
    assert!(must(core.me(unrelated), "unrelated local survives")["user"]["id"] == unrelated_id);
}

#[test]
#[ignore = "requires reviewed native IdP helper in RIAUTH_TEST_LASSO_IDP; one local lifecycle only"]
fn lasso_idp_redirect_post_source_lifecycle_replays_and_live_trust() {
    let budget = Instant::now() + Duration::from_secs(60);
    let bin = PathBuf::from(
        std::env::var_os("RIAUTH_TEST_LASSO_IDP").expect("select reviewed native IdP helper"),
    );
    let bin_meta = must(fs::symlink_metadata(&bin), "selected helper metadata");
    assert!(bin_meta.is_file() && !bin_meta.file_type().is_symlink() && bin_meta.nlink() == 1);
    let dir = must(tempfile::TempDir::new(), "own fresh private fixture");
    must(
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)),
        "private fixture mode",
    );
    let uid = must(fs::symlink_metadata(dir.path()), "own directory metadata").uid();
    assert!(bin_meta.uid() == uid && bin_meta.mode() & 0o022 == 0);
    let mut peer = NativePeer {
        bin,
        dir: dir.path().into(),
        uid,
        children: 0,
        budget,
    };
    let core = must(
        Core::initialize(
            Config {
                data_dir: dir.path().join("data"),
                issuer: "https://riauth.example.test".into(),
                ..Default::default()
            },
            new_user("native-admin", true),
        ),
        "fresh public Core initialize",
    );
    let admin = login(&core, "native-admin");
    let alice_id = text(
        &must(
            core.create_user(&admin, new_user("native-alice", false)),
            "public Alice creation",
        ),
        "id",
    );
    let unrelated_id = text(
        &must(
            core.create_user(&admin, new_user("native-unrelated", false)),
            "public unrelated creation",
        ),
        "id",
    );
    let alice = login(&core, "native-alice");
    let unrelated = login(&core, "native-unrelated");
    let sp_key = must(
        PKey::from_rsa(must(Rsa::generate(2048), "SP RSA")),
        "SP key",
    );
    let idp_key = must(
        PKey::from_rsa(must(Rsa::generate(2048), "IdP RSA")),
        "IdP key",
    );
    let other_key = must(
        PKey::from_rsa(must(Rsa::generate(2048), "unrelated RSA")),
        "unrelated key",
    );
    let (sp_cert, sp_der) = certificate(&sp_key, "native source SP");
    let (idp_cert, idp_der) = certificate(&idp_key, "native source IdP");
    let (other_cert, other_der) = certificate(&other_key, "unrelated native key");
    let sp_pem = must(
        String::from_utf8(must(sp_key.private_key_to_pem_pkcs8(), "SP PEM")),
        "SP PEM UTF8",
    );
    must(
        core.configure_key(
            &admin,
            KeyInput {
                id: "lasso-source-sp".into(),
                algorithm: "RS256".into(),
                private_key_pem: Some(sp_pem),
                remote_signer: None,
                kid: None,
            },
        ),
        "public SP signing domain",
    );
    let source = Source {
        id: SOURCE.into(),
        name: "Independent local Lasso IdP".into(),
        issuer: IDP.into(),
        client_id: SP.into(),
        authorization_endpoint: SSO.into(),
        token_endpoint: String::new(),
        token_endpoint_auth_method: ClientAuthMethod::None,
        jwks: Default::default(),
        scopes: BTreeSet::new(),
        enabled: true,
        auto_provision: false,
        groups: BTreeSet::new(),
        trusted_mfa_acr: BTreeSet::new(),
        allow_admin_login: false,
        oauth_profile: None,
        saml: Some(Settings {
            signing_key: "lasso-source-sp".into(),
            sp_certificate_pem: sp_cert,
            idp_certificates_pem: vec![idp_cert.clone()],
            name_id_format: NameIdFormat::Persistent,
            name_attribute: None,
            email_attribute: None,
            email_verified_attribute: None,
            require_encrypted_assertions: false,
            slo_redirect_url: None,
            slo_post_url: None,
        }),
    };
    must(
        core.source_put(
            &admin,
            SourceInput {
                source: source.clone(),
                client_secret: None,
            },
        ),
        "public source configure",
    );
    let sp_metadata = must(core.saml_source_metadata(SOURCE), "actual SP metadata");
    assert!(sp_metadata.contains(&sp_der));
    let paths: [PathBuf; 7] = [
        "idp-metadata.xml",
        "idp-key.pem",
        "idp-cert.pem",
        "sp-metadata.xml",
        "wrong-sp-metadata.xml",
        "other-key.pem",
        "other-cert.pem",
    ]
    .map(|name| peer.dir.join(name));
    private_write(&paths[0], idp_metadata(&idp_der).as_bytes());
    private_write(
        &paths[1],
        &must(idp_key.private_key_to_pem_pkcs8(), "IdP private PEM"),
    );
    private_write(&paths[2], idp_cert.as_bytes());
    private_write(&paths[3], sp_metadata.as_bytes());
    // Metadata fixture pin replacement only: entity/ACS unchanged, never a
    // successful protocol message or a re-signing substitute.
    private_write(
        &paths[4],
        sp_metadata.replace(&sp_der, &other_der).as_bytes(),
    );
    private_write(
        &paths[5],
        &must(other_key.private_key_to_pem_pkcs8(), "other private PEM"),
    );
    private_write(&paths[6], other_cert.as_bytes());
    live(budget);

    let a = start(&core, &peer, "a", Some(&alice));
    let before_refusal = snapshot(&core);
    let wrong = [
        paths[0].clone(),
        paths[1].clone(),
        paths[2].clone(),
        paths[4].clone(),
        a.query.clone(),
    ];
    let (refused, absent) = peer.run(&wrong, None); // child 1
    assert!(
        refused.status.code() == Some(1)
            && refused.diagnostic == Some(("lasso_login_process_authn_request_msg", 102)),
        "wrong-SP exit={:?} signal={:?} stage_code={:?}",
        refused.status.code(),
        refused.status.signal(),
        refused.diagnostic
    );
    assert!(refused.stdout.is_empty(), "no private refusal stdout");
    for output in [&absent.identity, &absent.session, &absent.post] {
        assert!(fs::symlink_metadata(output).is_err());
    }
    same(&core, &before_refusal);
    let inputs = |request: &Reservation| {
        [
            paths[0].clone(),
            paths[1].clone(),
            paths[2].clone(),
            paths[3].clone(),
            request.query.clone(),
        ]
    };
    let native_a = peer.accepted(&inputs(&a), None); // child 2, same unclaimed reservation
    let post_a = post(&core, &peer, &native_a, &a, &idp_der);
    let source_token_a = complete_link(&core, &peer, &a, &post_a, &alice_id);
    let links = must(core.source_links(&alice), "public links");
    let links = links.as_array().expect("links array");
    assert!(
        links.len() == 1 && links[0]["source"] == SOURCE && links[0]["subject"] == post_a.subject
    );
    let link = text(&links[0], "id");
    let before_unlink = snapshot(&core);
    refusal(core.source_unlink(&unrelated, &link), 403, None);
    same(&core, &before_unlink);
    refusal(core.source_unlink(&source_token_a, &link), 403, None);
    same(&core, &before_unlink);
    assert!(must(core.source_unlink(&alice, &link), "fresh local unlink")["unlinked"] == true);
    let unlinked = snapshot(&core);
    assert!(count(&unlinked, "source_links") == 0);
    assert!(
        audit_count(&unlinked, "source.unlink") == audit_count(&before_unlink, "source.unlink") + 1
    );
    refusal(core.me(&source_token_a), 401, None);
    locals(&core, &alice, &unrelated, &alice_id, &unrelated_id);
    refusal(core.source_unlink(&alice, &link), 403, None);
    same(&core, &unlinked);

    let b = start(&core, &peer, "b", Some(&alice));
    let native_b = peer.accepted(&inputs(&b), Some(&native_a.identity)); // child 3
    let post_b = post(&core, &peer, &native_b, &b, &idp_der);
    assert!(
        post_b.subject == post_a.subject
            && post_b.index != post_a.index
            && b.request_id != a.request_id
    );
    let source_token_b = complete_link(&core, &peer, &b, &post_b, &alice_id);
    let b_sid = text(
        &must(core.me(&source_token_b), "source B identity"),
        "session_id",
    );
    let c = start(&core, &peer, "c", None);
    let native_c = peer.accepted(&inputs(&c), Some(&native_b.identity)); // child 4, before trust withdrawal
    let post_c = post(&core, &peer, &native_c, &c, &idp_der);
    assert!(post_c.subject == post_b.subject && post_c.index != post_b.index);
    let mut withdrawn = source.clone();
    withdrawn
        .saml
        .as_mut()
        .expect("SAML source")
        .idp_certificates_pem = vec![other_cert.clone()];
    must(
        core.source_put(
            &admin,
            SourceInput {
                source: withdrawn,
                client_secret: None,
            },
        ),
        "public trust withdrawal",
    );
    let retired = snapshot(&core);
    assert!(retired[&format!("sessions/{b_sid}")]["revoked"] == true);
    assert!(retired[&format!("sessions/{b_sid}")]["identity"]["source"]["pin_retired"] == true);
    refusal(core.me(&source_token_b), 401, None);
    locals(&core, &alice, &unrelated, &alice_id, &unrelated_id);
    let before_stale = snapshot(&core);
    refusal(
        callback(&core, &post_c, &c),
        400,
        Some("SAML source changed; restart login"),
    );
    sealed_denial(&core, &before_stale, &c.relay);
    must(
        core.source_put(
            &admin,
            SourceInput {
                source,
                client_secret: None,
            },
        ),
        "public original pin restore",
    );
    let restored = snapshot(&core);
    refusal(
        callback(&core, &post_c, &c),
        400,
        Some("SAML source request expired or already used"),
    );
    same(&core, &restored);
    refusal(finish(&core, &c, true), 401, None);
    same(&core, &restored);
    refusal(core.me(&source_token_b), 401, None);
    same(&core, &restored);

    let d = start(&core, &peer, "d", None);
    let other_metadata = peer.dir.join("other-idp-metadata.xml");
    private_write(&other_metadata, idp_metadata(&other_der).as_bytes());
    let wrong_idp = [
        other_metadata,
        paths[5].clone(),
        paths[6].clone(),
        paths[3].clone(),
        d.query.clone(),
    ];
    let native_d = peer.accepted(&wrong_idp, Some(&native_c.identity)); // child 5
    let post_d = post(&core, &peer, &native_d, &d, &other_der);
    assert!(post_d.subject == post_a.subject);
    let before_bad_signature = snapshot(&core);
    let rejected = must(
        callback(&core, &post_d, &d),
        "signature denial sealed callback body",
    );
    assert!(rejected["completed"] == false);
    sealed_denial(&core, &before_bad_signature, &d.relay);
    let failed = snapshot(&core);
    refusal(finish(&core, &d, true), 401, None);
    same(&core, &failed);
    refusal(
        callback(&core, &post_d, &d),
        400,
        Some("SAML source request expired or already used"),
    );
    same(&core, &failed);
    refusal(finish(&core, &d, true), 401, None);
    same(&core, &failed);
    locals(&core, &alice, &unrelated, &alice_id, &unrelated_id);
    assert!(peer.children == 5, "exact designed helper count");
    live(budget);
    // Child guards reaped each invocation; Core drops before its owned TempDir.
}
