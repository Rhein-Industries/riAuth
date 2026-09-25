use super::saml_source_tests::Upstream;
use super::*;
use base64::engine::general_purpose::STANDARD;
use openssl::{
    hash::MessageDigest,
    pkey::PKey,
    rsa::Rsa,
    sign::{Signer, Verifier},
};
use riauth::{
    saml::{NameIdFormat, Reply},
    source::{Finish, Source, SourceInput},
};
use std::{
    io::{Read, Write},
    path::Path,
};
const P: &str = "urn:oasis:names:tc:SAML:2.0:protocol";
const A: &str = "urn:oasis:names:tc:SAML:2.0:assertion";
const ALG: &str = "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256";
fn at(value: u64) -> String {
    time::OffsetDateTime::from_unix_timestamp(value as i64)
        .unwrap()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap()
}
fn peer(binary: Option<&Path>) -> Upstream {
    let key = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();
    let cert = saml_tests::cert(&key, "SAML logout peer");
    let dir = tempfile::tempdir().unwrap();
    riauth::config::write_private(
        &dir.path().join("idp.key"),
        &key.private_key_to_pem_pkcs8().unwrap(),
        false,
    )
    .unwrap();
    Upstream {
        key,
        cert,
        xmlsec: binary.map(Path::to_owned),
        dir,
    }
}
fn message(peer: &Upstream, xml: &str, relay: &str, field: &str, post: bool) -> String {
    if post {
        return url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs([
                (field, STANDARD.encode(peer.sign(xml, true)).as_str()),
                ("RelayState", relay),
            ])
            .finish();
    }
    let mut compressed =
        flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
    compressed.write_all(xml.as_bytes()).unwrap();
    let mut query = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs([
            (
                field,
                STANDARD.encode(compressed.finish().unwrap()).as_str(),
            ),
            ("RelayState", relay),
            ("SigAlg", ALG),
        ])
        .finish();
    let mut signer = Signer::new(MessageDigest::sha256(), &peer.key).unwrap();
    signer.update(query.as_bytes()).unwrap();
    query.push('&');
    query.push_str(
        &url::form_urlencoded::Serializer::new(String::new())
            .append_pair("Signature", &STANDARD.encode(signer.sign_to_vec().unwrap()))
            .finish(),
    );
    query
}
fn decode(reply: Reply, cert: &str, binary: Option<&Path>) -> (String, String, bool) {
    match reply {
        Reply::Redirect(uri) => {
            let uri = url::Url::parse(&uri).unwrap();
            let query = uri.query().unwrap();
            let (signed, _) = query.rsplit_once("&Signature=").unwrap();
            let fields = uri
                .query_pairs()
                .into_owned()
                .collect::<std::collections::BTreeMap<_, _>>();
            let cert = openssl::x509::X509::from_pem(cert.as_bytes()).unwrap();
            let key = cert.public_key().unwrap();
            let mut verify = Verifier::new(MessageDigest::sha256(), &key).unwrap();
            verify.update(signed.as_bytes()).unwrap();
            assert!(
                verify
                    .verify(&STANDARD.decode(&fields["Signature"]).unwrap())
                    .unwrap()
            );
            let mut xml = String::new();
            flate2::read::DeflateDecoder::new(
                STANDARD
                    .decode(
                        fields
                            .get("SAMLRequest")
                            .or_else(|| fields.get("SAMLResponse"))
                            .unwrap(),
                    )
                    .unwrap()
                    .as_slice(),
            )
            .read_to_string(&mut xml)
            .unwrap();
            (
                xml,
                fields.get("RelayState").cloned().unwrap_or_default(),
                false,
            )
        }
        Reply::Post { fields, .. } => {
            let fields = fields
                .into_iter()
                .collect::<std::collections::BTreeMap<_, _>>();
            let xml = String::from_utf8(
                STANDARD
                    .decode(
                        fields
                            .get("SAMLRequest")
                            .or_else(|| fields.get("SAMLResponse"))
                            .unwrap(),
                    )
                    .unwrap(),
            )
            .unwrap();
            if let Some(binary) = binary {
                let dir = tempfile::tempdir().unwrap();
                let file = dir.path().join("message.xml");
                let certificate = dir.path().join("signer.pem");
                std::fs::write(&file, &xml).unwrap();
                std::fs::write(&certificate, cert).unwrap();
                saml_tests::xmlsec(
                    binary,
                    &[
                        "--verify",
                        "--trusted-pem",
                        certificate.to_str().unwrap(),
                        "--id-attr:ID",
                        "LogoutRequest",
                        "--id-attr:ID",
                        "LogoutResponse",
                        file.to_str().unwrap(),
                    ],
                );
            } else {
                assert!(
                    risaml::crypto::verify_signature(&xml, &[cert.into()])
                        .unwrap()
                        .0
                );
            }
            (
                xml,
                fields.get("RelayState").cloned().unwrap_or_default(),
                true,
            )
        }
        _ => panic!("expected a signed SAML message"),
    }
}
fn ticket(reply: Reply) -> String {
    let uri = match reply {
        Reply::Redirect(uri) => uri,
        Reply::LogoutPage(v) => text(&v, "redirect_uri"),
        _ => panic!("expected logout continuation"),
    };
    assert!(uri.starts_with("http://localhost:9000/saml/logout/"));
    uri.rsplit('/').next().unwrap().into()
}
fn provider(f: &Fixture, id: &str, peer: &Upstream, cert: &str, post: bool) {
    let settings = riauth::saml::Settings {
        sp_entity_id: format!("urn:test:{id}"),
        acs_urls: vec![format!("https://{id}.example.test/acs")],
        acs_indices: Default::default(),
        idp_entity_id: Some(format!("urn:test:riauth:{id}")),
        idp_certificate_pem: cert.into(),
        sp_certificates_pem: vec![peer.cert.clone()],
        encryption_certificate_pem: None,
        name_id_format: NameIdFormat::Persistent,
        attributes: vec![],
        slo_redirect_url: (!post).then(|| format!("https://{id}.example.test/slo")),
        slo_post_url: post.then(|| format!("https://{id}.example.test/slo")),
        idp_initiated: true,
        default_relay_state: None,
        assertion_ttl: 120,
    };
    f.core
        .create_client(
            &f.admin,
            NewClient {
                client_id: id.into(),
                name: id.into(),
                confidential: false,
                redirect_uris: vec![],
                scopes: strings(&["openid", "saml"]),
                allowed_groups: Default::default(),
                require_mfa: false,
                service: false,
                settings: riauth::model::ProviderSettings {
                    saml: Some(settings),
                    ..Default::default()
                },
            },
        )
        .unwrap();
}
fn login_sp(f: &Fixture, id: &str, session: &str) -> (String, String) {
    let Reply::Waiting(wait) = f.core.saml_initiate(id, None).unwrap() else {
        panic!("expected handoff")
    };
    let code = text(&wait.body, "user_code");
    let pending = wait.refresh.unwrap().rsplit('/').next().unwrap().to_owned();
    let cookie = wait.cookies[0]
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1
        .to_owned();
    f.core
        .saml_decide(
            session,
            riauth::browser::BrowserDecision {
                code,
                approve: true,
                transaction_id: None,
                remember: false,
            },
        )
        .unwrap();
    let Reply::Post { fields, .. } = f.core.saml_resume(&pending, Some(&cookie)).unwrap() else {
        panic!("expected assertion")
    };
    let xml = String::from_utf8(STANDARD.decode(&fields[0].1).unwrap()).unwrap();
    let doc = roxmltree::Document::parse(&xml).unwrap();
    (
        doc.descendants()
            .find(|n| n.has_tag_name((A, "NameID")))
            .unwrap()
            .text()
            .unwrap()
            .into(),
        doc.descendants()
            .find_map(|n| n.attribute("SessionIndex"))
            .unwrap()
            .into(),
    )
}
fn response(xml: &str, peer: &str, callback: &str, status: &str) -> String {
    let doc = roxmltree::Document::parse(xml).unwrap();
    let id = doc.root_element().attribute("ID").unwrap();
    format!(
        r#"<samlp:LogoutResponse xmlns:samlp="{P}" xmlns:saml="{A}" ID="_{}" Version="2.0" IssueInstant="{}" Destination="{callback}" InResponseTo="{id}"><saml:Issuer>{peer}</saml:Issuer><samlp:Status><samlp:StatusCode Value="urn:oasis:names:tc:SAML:2.0:status:{status}"/></samlp:Status></samlp:LogoutResponse>"#,
        crypto::id(),
        at(now())
    )
}
fn request(
    subject: &str,
    index: &str,
    issuer: &str,
    destination: &str,
    nq: &str,
    spq: &str,
) -> String {
    format!(
        r#"<samlp:LogoutRequest xmlns:samlp="{P}" xmlns:saml="{A}" ID="_{}" Version="2.0" IssueInstant="{}" Destination="{destination}"><saml:Issuer>{issuer}</saml:Issuer><saml:NameID Format="{}" NameQualifier="{nq}" SPNameQualifier="{spq}">{subject}</saml:NameID><samlp:SessionIndex>{index}</samlp:SessionIndex></samlp:LogoutRequest>"#,
        crypto::id(),
        at(now()),
        NameIdFormat::Persistent.uri()
    )
}
fn source_login(f: &Fixture, source: &Source, upstream: &Upstream, link: Option<&str>) -> String {
    let (state, id, credential) = saml_source_tests::begin(f, source, link);
    let xml = upstream.response(
        source,
        &f.core.saml_source_callback_url(&source.id),
        &id,
        None,
        false,
    );
    assert_eq!(
        f.core
            .saml_source_callback(
                &source.id,
                vec![
                    ("RelayState".into(), state),
                    ("SAMLResponse".into(), STANDARD.encode(xml))
                ]
            )
            .unwrap()["completed"],
        true
    );
    text(
        &f.core
            .source_finish(Finish {
                credential,
                approve: true,
                otp: None,
            })
            .unwrap(),
        "session_token",
    )
}
#[test]
fn saml_logout_multiple_sps_source_binding_revocation_retry_and_partial() {
    exercise(None)
}
#[test]
#[ignore = "requires RIAUTH_TEST_XMLSEC"]
fn saml_logout_independent_xmlsec() {
    exercise(Some(Path::new(
        &std::env::var("RIAUTH_TEST_XMLSEC").unwrap(),
    )))
}
fn exercise(binary: Option<&Path>) {
    let f = Fixture::new();
    let alice = f.user("logout-alice");
    let bob = f.user("logout-bob");
    let sp1 = peer(binary);
    let sp2 = peer(binary);
    let upstream = peer(binary);
    let keys: crypto::Keys = f.core.store.get("meta", "keys").unwrap().unwrap();
    let local = PKey::private_key_from_pem(keys.active.pem.as_bytes()).unwrap();
    let cert = saml_tests::cert(&local, "riAuth SLO");
    provider(&f, "first", &sp1, &cert, false);
    provider(&f, "second", &sp2, &cert, true);
    let source = Source {
        id: "upstream".into(),
        name: "SAML upstream".into(),
        issuer: "urn:test:upstream".into(),
        authorization_endpoint: "https://upstream.example.test/sso".into(),
        token_endpoint: "".into(),
        client_id: "urn:test:riauth-source".into(),
        token_endpoint_auth_method: riauth::jose::ClientAuthMethod::None,
        jwks: Default::default(),
        scopes: Default::default(),
        enabled: true,
        auto_provision: false,
        groups: Default::default(),
        trusted_mfa_acr: Default::default(),
        allow_admin_login: false,
        oauth_profile: None,
        saml: Some(riauth::source::saml::Settings {
            signing_key: "signing".into(),
            sp_certificate_pem: cert.clone(),
            idp_certificates_pem: vec![upstream.cert.clone()],
            name_id_format: NameIdFormat::Persistent,
            name_attribute: None,
            email_attribute: None,
            email_verified_attribute: None,
            require_encrypted_assertions: false,
            slo_redirect_url: None,
            slo_post_url: Some("https://upstream.example.test/slo".into()),
        }),
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
    assert!(
        f.core
            .saml_source_metadata(&source.id)
            .unwrap()
            .contains("/saml/sources/upstream/slo")
    );
    let session = source_login(&f, &source, &upstream, Some(&alice));
    let (name1, index1) = login_sp(&f, "first", &session);
    let (name2, index2) = login_sp(&f, "second", &session);
    let bob_sp = login_sp(&f, "first", &bob);
    let revoked = f.core.logout(&session).unwrap();
    assert!(f.core.me(&session).is_err());
    assert!(f.core.me(&bob).is_ok());
    let flow = revoked["saml_logout_url"]
        .as_str()
        .unwrap()
        .rsplit('/')
        .next()
        .unwrap()
        .to_owned();
    let mut seen = BTreeSet::new();
    for _ in 0..3 {
        let (xml, relay, post) = decode(f.core.saml_logout_next(&flow).unwrap(), &cert, binary);
        let (retry, relay_retry, _) =
            decode(f.core.saml_logout_next(&flow).unwrap(), &cert, binary);
        assert_eq!(xml, retry);
        assert_eq!(relay, relay_retry);
        assert_eq!(relay, flow);
        let doc = roxmltree::Document::parse(&xml).unwrap();
        let root = doc.root_element();
        let dest = root.attribute("Destination").unwrap();
        let (id, signer, callback, peer_id, expected_name, expected_index) =
            if dest.contains("first.") {
                (
                    "first",
                    &sp1,
                    "http://localhost:9000/saml/first/sso",
                    "urn:test:first",
                    name1.as_str(),
                    index1.as_str(),
                )
            } else if dest.contains("second.") {
                (
                    "second",
                    &sp2,
                    "http://localhost:9000/saml/second/sso",
                    "urn:test:second",
                    name2.as_str(),
                    index2.as_str(),
                )
            } else {
                (
                    "upstream",
                    &upstream,
                    "http://localhost:9000/saml/sources/upstream/slo",
                    "urn:test:upstream",
                    "opaque-subject",
                    "upstream-session",
                )
            };
        assert!(seen.insert(id));
        assert_eq!(
            doc.descendants()
                .find(|n| n.has_tag_name((A, "NameID")))
                .unwrap()
                .text(),
            Some(expected_name)
        );
        assert_eq!(
            doc.descendants()
                .find(|n| n.has_tag_name((P, "SessionIndex")))
                .unwrap()
                .text(),
            Some(expected_index)
        );
        assert!(root.attribute("NotOnOrAfter").is_some());
        let correct = response(&xml, peer_id, callback, "Success");
        let call = |body: &str| {
            if id == "upstream" {
                f.core.saml_source_logout(id, body, post)
            } else {
                f.core.saml_start(id, body, post, None)
            }
        };
        for invalid in [
            correct.replace("InResponseTo=", "WrongRequest="),
            correct.replace(peer_id, "urn:wrong-peer"),
            correct.replace(callback, "https://attacker.example.test/slo"),
        ] {
            assert!(call(&message(signer, &invalid, &relay, "SAMLResponse", post)).is_err());
        }
        assert!(
            call(&message(
                signer,
                &correct,
                &crypto::random_token(""),
                "SAMLResponse",
                post
            ))
            .is_err()
        );
        let body = message(signer, &correct, &relay, "SAMLResponse", post);
        assert_eq!(ticket(call(&body).unwrap()), flow);
        assert!(call(&body).is_err());
    }
    let Reply::LogoutPage(done) = f.core.saml_logout_next(&flow).unwrap() else {
        panic!("expected completion")
    };
    assert_eq!(done["status"], "complete");
    assert_eq!(done["confirmed"], 3);
    assert_eq!(f.core.saml_logout_status(&flow).unwrap()["remaining"], 0);
    // A signed IdP request revokes exactly its upstream subject/index and fans out
    // downstream; it must never send another request to that initiating source.
    let fresh = source_login(&f, &source, &upstream, None);
    login_sp(&f, "second", &fresh);
    let upstream_request = request(
        "opaque-subject",
        "upstream-session",
        &source.issuer,
        "http://localhost:9000/saml/sources/upstream/slo",
        &source.issuer,
        &source.client_id,
    );
    let invalid = upstream_request.replace("opaque-subject", "somebody-else");
    assert!(
        f.core
            .saml_source_logout(
                "upstream",
                &message(&upstream, &invalid, "original-state", "SAMLRequest", true),
                true
            )
            .is_err()
    );
    assert!(f.core.me(&fresh).is_ok());
    let raw = message(
        &upstream,
        &upstream_request,
        "original-state",
        "SAMLRequest",
        true,
    );
    let flow = ticket(f.core.saml_source_logout("upstream", &raw, true).unwrap());
    assert!(f.core.me(&fresh).is_err());
    assert!(f.core.me(&bob).is_ok());
    assert!(f.core.saml_source_logout("upstream", &raw, true).is_err());
    // Earlier revoked local sessions sharing this upstream session participate too.
    // Each signed failure advances the coordinator and produces Success/PartialLogout.
    while f.core.saml_logout_status(&flow).unwrap()["remaining"]
        .as_u64()
        .unwrap()
        > 0
    {
        let (xml, relay, post) = decode(f.core.saml_logout_next(&flow).unwrap(), &cert, binary);
        assert!(!xml.contains("Destination=\"https://upstream."));
        let is_first = xml.contains("Destination=\"https://first.");
        let id = if is_first { "first" } else { "second" };
        let signer = if is_first { &sp1 } else { &sp2 };
        let xml = response(
            &xml,
            &format!("urn:test:{id}"),
            &format!("http://localhost:9000/saml/{id}/sso"),
            "Responder",
        );
        f.core
            .saml_start(
                id,
                &message(signer, &xml, &relay, "SAMLResponse", post),
                post,
                None,
            )
            .unwrap();
    }
    let (final_xml, relay, post) = decode(f.core.saml_logout_next(&flow).unwrap(), &cert, binary);
    assert!(post);
    assert_eq!(relay, "original-state");
    assert!(final_xml.contains("status:Success\"") && final_xml.contains("status:PartialLogout\""));
    // SP initiation must resolve all indices before revocation and preserve the
    // initiating response while other SPs time out or change trust configuration.
    let own = f
        .core
        .login("logout-alice".into(), PASSWORD.into(), None)
        .unwrap();
    let own = text(&own, "session_token");
    let (name, index) = login_sp(&f, "first", &own);
    login_sp(&f, "second", &own);
    let wrong = request(
        &name,
        &bob_sp.1,
        "urn:test:first",
        "http://localhost:9000/saml/first/sso",
        "urn:test:riauth:first",
        "urn:test:first",
    );
    assert!(
        f.core
            .saml_start(
                "first",
                &message(&sp1, &wrong, "sp-state", "SAMLRequest", false),
                false,
                None
            )
            .is_err()
    );
    assert!(f.core.me(&own).is_ok());
    let request = request(
        &name,
        &index,
        "urn:test:first",
        "http://localhost:9000/saml/first/sso",
        "urn:test:riauth:first",
        "urn:test:first",
    );
    let flow = ticket(
        f.core
            .saml_start(
                "first",
                &message(&sp1, &request, "sp-state", "SAMLRequest", false),
                false,
                None,
            )
            .unwrap(),
    );
    let (out, relay, post) = decode(f.core.saml_logout_next(&flow).unwrap(), &cert, binary);
    assert!(post);
    assert!(out.contains("https://second."));
    // Advance the durable deadline directly; this verifies expiry without sleeping.
    f.core
        .store
        .write(|tx| {
            let key = crypto::digest(&flow);
            let mut value: Value = tx.get("saml_logout_flows", &key)?.unwrap();
            value["pending"]["deadline"] = json!(now() - 1);
            tx.put("saml_logout_flows", &key, &value)
        })
        .unwrap();
    let late = response(
        &out,
        "urn:test:second",
        "http://localhost:9000/saml/second/sso",
        "Success",
    );
    assert!(
        f.core
            .saml_start(
                "second",
                &message(&sp2, &late, &relay, "SAMLResponse", true),
                true,
                None
            )
            .is_err()
    );
    let (final_xml, relay, _) = decode(f.core.saml_logout_next(&flow).unwrap(), &cert, binary);
    assert_eq!(relay, "sp-state");
    assert!(final_xml.contains("status:PartialLogout"));
    assert!(f.core.me(&bob).is_ok());
    let fresh = text(
        &f.core
            .login("logout-alice".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    login_sp(&f, "second", &fresh);
    let result = f.core.logout(&fresh).unwrap();
    let flow = result["saml_logout_url"]
        .as_str()
        .unwrap()
        .rsplit('/')
        .next()
        .unwrap();
    f.core
        .store
        .write(|tx| {
            let mut client: riauth::model::Client = tx.get("clients", "second")?.unwrap();
            client.name = "Changed while logout was pending".into();
            tx.put("clients", "second", &client)
        })
        .unwrap();
    let Reply::LogoutPage(done) = f.core.saml_logout_next(flow).unwrap() else {
        panic!("changed trust must not receive old identity")
    };
    assert_eq!(done["status"], "partial_logout");
    assert_eq!(done["failed"], 1);
}

#[tokio::test]
async fn saml_logout_oidc_continuation_http_and_confirmation() {
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    let f = Fixture::new();
    let alice = f.user("bridge-alice");
    let sp = peer(None);
    let keys: crypto::Keys = f.core.store.get("meta", "keys").unwrap().unwrap();
    let local = PKey::private_key_from_pem(keys.active.pem.as_bytes()).unwrap();
    let cert = saml_tests::cert(&local, "SAML bridge");
    provider(&f, "bridge", &sp, &cert, false);
    f.client("oidc", false);
    f.core
        .update_client(
            &f.admin,
            "oidc",
            ClientPatch {
                settings: Some(ProviderSettings {
                    post_logout_redirect_uris: vec!["https://app.example.test/finished".into()],
                    frontchannel_logout_uri: Some("https://app.example.test/front".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();
    let tokens = f.tokens("oidc", &alice, None);
    login_sp(&f, "bridge", &alice);
    // An ended RP grant is not proof that the parent session was revoked.
    f.core.revoke_consent(&alice, "oidc").unwrap();
    assert!(f.core.me(&alice).is_ok());
    let request = riauth::logout::LogoutRequest {
        id_token_hint: Some(text(&tokens, "id_token")),
        client_id: Some("oidc".into()),
        post_logout_redirect_uri: Some("https://app.example.test/finished".into()),
        state: Some("preserved".into()),
    };
    let result = f
        .core
        .end_session(request.clone(), None, Some(&alice))
        .unwrap();
    assert!(f.core.me(&alice).is_err());
    assert_eq!(result["frontchannel_urls"].as_array().unwrap().len(), 1);
    assert_eq!(
        f.core.end_session(request, None, Some(&alice)).unwrap()["redirect_uri"],
        result["redirect_uri"]
    );
    let flow = result["redirect_uri"]
        .as_str()
        .unwrap()
        .rsplit('/')
        .next()
        .unwrap();
    let app = riauth::api::router(f.core.clone());
    let status = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri(format!("/saml/logout/{flow}/status"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(status.status(), 200);
    let status: Value =
        serde_json::from_slice(&status.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(status["remaining"], 1);
    let next = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri(format!("/saml/logout/{flow}"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(next.status().is_redirection());
    let (xml, relay, _) = decode(
        Reply::Redirect(next.headers()["location"].to_str().unwrap().into()),
        &cert,
        None,
    );
    let correct = response(
        &xml,
        "urn:test:bridge",
        "http://localhost:9000/saml/bridge/sso",
        "Success",
    );
    let callback = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri(format!(
                    "/saml/bridge/sso?{}",
                    message(&sp, &correct, &relay, "SAMLResponse", false)
                ))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(callback.status().is_redirection());
    let final_reply = app
        .oneshot(
            axum::http::Request::builder()
                .uri(format!("/saml/logout/{flow}"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        final_reply.headers()["location"],
        "https://app.example.test/finished?state=preserved"
    );
    let fresh = text(
        &f.core
            .login("bridge-alice".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    login_sp(&f, "bridge", &fresh);
    let tokens = f.tokens("oidc", &fresh, None);
    let confirmation = f
        .core
        .end_session(
            riauth::logout::LogoutRequest {
                id_token_hint: Some(text(&tokens, "id_token")),
                client_id: Some("oidc".into()),
                ..Default::default()
            },
            None,
            None,
        )
        .unwrap();
    f.core
        .logout_request_decide(&fresh, &text(&confirmation, "user_code"), true)
        .unwrap();
    assert!(f.core.me(&fresh).is_err());
    let id = confirmation["resume_uri"]
        .as_str()
        .unwrap()
        .rsplit('/')
        .next()
        .unwrap();
    let binding = confirmation["set_cookie"]
        .as_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1;
    let resumed = f.core.logout_request_resume(id, Some(binding)).unwrap();
    assert!(
        resumed["redirect_uri"]
            .as_str()
            .unwrap()
            .contains("/saml/logout/")
    );
}
