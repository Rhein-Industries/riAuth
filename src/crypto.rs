use crate::error::{Error, Result};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use aws_lc_rs::{
    encoding::{AsDer, Pkcs8V1Der},
    rsa::{KeyPair as RsaKeyPair, KeySize, PublicKeyComponents},
    signature::KeyPair,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use rand::{Rng, RngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use totp_rs::{Algorithm as TotpAlgorithm, Secret, TOTP};

#[cfg(feature = "test-support")]
thread_local! { static TEST_TIME: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) }; }
/// A thread-local test clock; never shared between concurrently running tests.
#[cfg(feature = "test-support")]
pub fn with_test_time<T>(at: u64, f: impl FnOnce() -> T) -> T {
    struct Reset(Option<u64>);
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_TIME.with(|clock| clock.set(self.0));
        }
    }
    let _reset = Reset(TEST_TIME.with(|clock| clock.replace(Some(at))));
    f()
}
#[cfg(feature = "test-support")]
pub fn set_test_time(at: u64) {
    TEST_TIME.with(|clock| {
        assert!(
            clock.get().is_some(),
            "set_test_time requires with_test_time"
        );
        clock.set(Some(at));
    });
}

pub fn now() -> u64 {
    #[cfg(feature = "test-support")]
    if let Some(at) = TEST_TIME.with(|clock| clock.get()) {
        return at;
    }
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before UNIX epoch")
        .as_secs()
}
pub fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
pub fn random_token(prefix: &str) -> String {
    let mut bytes = [0; 32];
    OsRng.fill_bytes(&mut bytes);
    format!("{prefix}{}", URL_SAFE_NO_PAD.encode(bytes))
}
pub fn digest(value: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(value.as_bytes()))
}
pub fn constant_eq(a: &str, b: &str) -> bool {
    bool::from(a.as_bytes().ct_eq(b.as_bytes()))
}

pub fn seal(key: &[u8; 32], context: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
    use aws_lc_rs::aead::{AES_256_GCM, Aad, LessSafeKey, Nonce, UnboundKey};
    let key = LessSafeKey::new(UnboundKey::new(&AES_256_GCM, key).map_err(Error::internal)?);
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    let mut body = plaintext.to_vec();
    key.seal_in_place_append_tag(
        Nonce::assume_unique_for_key(nonce),
        Aad::from(context),
        &mut body,
    )
    .map_err(Error::internal)?;
    let mut output = b"RIAUTH-AEAD1".to_vec();
    output.extend(nonce);
    output.extend(body);
    Ok(output)
}
pub fn unseal(
    key: &[u8; 32],
    context: &[u8],
    ciphertext: &[u8],
) -> Result<zeroize::Zeroizing<Vec<u8>>> {
    use aws_lc_rs::aead::{AES_256_GCM, Aad, LessSafeKey, Nonce, UnboundKey};
    let body = ciphertext
        .strip_prefix(b"RIAUTH-AEAD1")
        .filter(|b| b.len() >= 28)
        .ok_or_else(|| Error::bad("Invalid encrypted data format"))?;
    let nonce: [u8; 12] = body[..12].try_into().map_err(Error::internal)?;
    let key = LessSafeKey::new(UnboundKey::new(&AES_256_GCM, key).map_err(Error::internal)?);
    let mut plaintext = zeroize::Zeroizing::new(body[12..].to_vec());
    let len = key
        .open_in_place(
            Nonce::assume_unique_for_key(nonce),
            Aad::from(context),
            &mut plaintext,
        )
        .map_err(|_| Error::bad("Encrypted data authentication failed: wrong key or damaged data"))?
        .len();
    plaintext.truncate(len);
    Ok(plaintext)
}
pub fn read_key(path: &std::path::Path) -> Result<zeroize::Zeroizing<[u8; 32]>> {
    let bytes = crate::config::read_private_secret(path, 128)
        .map_err(|_| Error::bad("Encryption key must be a private file of at most 128 bytes"))?;
    let decoded = zeroize::Zeroizing::new(
        URL_SAFE_NO_PAD
            .decode(bytes.trim())
            .map_err(|_| Error::bad("Encryption key must be base64url without padding"))?,
    );
    Ok(zeroize::Zeroizing::new(
        decoded
            .as_slice()
            .try_into()
            .map_err(|_| Error::bad("Encryption key must contain 32 random bytes"))?,
    ))
}
pub fn password_hash(password: &str) -> Result<String> {
    if password.len() < 12 || password.len() > 1024 {
        return Err(Error::bad("Passwords must be between 12 and 1024 bytes"));
    }
    Argon2::default()
        .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
        .map(|hash| hash.to_string())
        .map_err(Error::internal)
}
pub fn password_matches(password: &str, hash: &str) -> bool {
    if password.len() > 1024 {
        return false;
    }
    if validate_imported_hash(hash).is_err() {
        return false;
    }
    if hash.starts_with("pbkdf2_sha256$") {
        let parts: Vec<_> = hash.split('$').collect();
        let Ok(rounds) = parts[1].parse::<std::num::NonZeroU32>() else {
            return false;
        };
        let Ok(expected) = base64::engine::general_purpose::STANDARD.decode(parts[3]) else {
            return false;
        };
        return aws_lc_rs::pbkdf2::verify(
            aws_lc_rs::pbkdf2::PBKDF2_HMAC_SHA256,
            rounds,
            parts[2].as_bytes(),
            password.as_bytes(),
            &expected,
        )
        .is_ok();
    }
    PasswordHash::new(hash).is_ok_and(|parsed| {
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    })
}

pub fn validate_imported_hash(hash: &str) -> Result<String> {
    if hash.len() > 1024 {
        return Err(Error::bad("Password hash exceeds supported size"));
    }
    let normalized = hash
        .strip_prefix("argon2$")
        .map(|s| format!("${s}"))
        .unwrap_or_else(|| hash.to_owned());
    let valid = if normalized.starts_with("pbkdf2_sha256$") {
        let parts: Vec<_> = normalized.split('$').collect();
        parts.len() == 4
            && parts[1]
                .parse::<u32>()
                .is_ok_and(|n| (1000..=2_000_000).contains(&n))
            && !parts[2].is_empty()
            && parts[2].len() <= 128
            && parts[2].is_ascii()
            && base64::engine::general_purpose::STANDARD
                .decode(parts[3])
                .is_ok_and(|b| b.len() == 32)
    } else {
        PasswordHash::new(&normalized).is_ok_and(|h| {
            ["argon2id", "argon2i"].contains(&h.algorithm.as_str())
                && h.version == Some(19)
                && h.params
                    .get_decimal("m")
                    .is_some_and(|v| (8..=262_144).contains(&v))
                && h.params
                    .get_decimal("t")
                    .is_some_and(|v| (1..=10).contains(&v))
                && h.params
                    .get_decimal("p")
                    .is_some_and(|v| (1..=16).contains(&v))
                && h.hash.is_some()
                && h.salt.is_some()
        })
    };
    if !valid {
        return Err(Error::bad(
            "Unsupported password hash or excessive cost; supported: Argon2id/i v19 and Django PBKDF2-SHA256",
        ));
    }
    Ok(normalized)
}

pub fn upgrade_password_hash(password: &str, old: &str) -> Result<Option<String>> {
    // Imported Argon2id costs other than the default are rehashed, so their timing stops standing out.
    let nondefault = old.starts_with("$argon2id$")
        && PasswordHash::new(old).is_ok_and(|h| {
            let default = argon2::Params::default();
            [
                ("m", default.m_cost()),
                ("t", default.t_cost()),
                ("p", default.p_cost()),
            ]
            .into_iter()
            .any(|(name, value)| h.params.get_decimal(name) != Some(value))
        });
    if !old.starts_with("pbkdf2_sha256$") && !old.starts_with("$argon2i$") && !nondefault {
        return Ok(None);
    }
    // Preserve legacy passwords on first login, including lengths below the new-password policy.
    Argon2::default()
        .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
        .map(|hash| Some(hash.to_string()))
        .map_err(Error::internal)
}
pub fn user_code() -> String {
    const ALPHABET: &[u8] = b"BCDFGHJKLMNPQRSTVWXYZ23456789";
    let raw: String = (0..10)
        .map(|_| ALPHABET[OsRng.gen_range(0..ALPHABET.len())] as char)
        .collect();
    format!("{}-{}", &raw[..5], &raw[5..])
}
pub fn normalize_code(code: &str) -> Result<String> {
    let normalized: String = code
        .chars()
        .filter(|c| *c != '-' && *c != ' ')
        .collect::<String>()
        .to_ascii_uppercase();
    if normalized.len() != 10 || !normalized.bytes().all(|b| b.is_ascii_alphanumeric()) {
        return Err(Error::bad("Invalid user code"));
    }
    Ok(normalized)
}
pub fn totp(secret: &str, username: &str) -> Result<TOTP> {
    totp_with(secret, username, &Default::default())
}
pub fn totp_with(
    secret: &str,
    username: &str,
    settings: &crate::authenticator::TotpSettings,
) -> Result<TOTP> {
    settings.validate()?;
    let bytes = Secret::Encoded(secret.to_owned())
        .to_bytes()
        .map_err(Error::internal)?;
    TOTP::new(
        match settings.algorithm.as_str() {
            "SHA256" => TotpAlgorithm::SHA256,
            "SHA512" => TotpAlgorithm::SHA512,
            _ => TotpAlgorithm::SHA1,
        },
        settings.digits,
        1,
        settings.period,
        bytes,
        Some("riAuth".into()),
        username.into(),
    )
    .map_err(Error::internal)
}
pub fn totp_secret() -> String {
    let mut bytes = vec![0; 20];
    OsRng.fill_bytes(&mut bytes);
    Secret::Raw(bytes).to_encoded().to_string()
}
pub fn totp_step(
    secret: &str,
    username: &str,
    code: &str,
    at: u64,
    last_step: Option<u64>,
) -> Result<Option<u64>> {
    totp_step_with(secret, username, code, at, last_step, &Default::default())
}
pub fn totp_step_with(
    secret: &str,
    username: &str,
    code: &str,
    at: u64,
    last_step: Option<u64>,
    settings: &crate::authenticator::TotpSettings,
) -> Result<Option<u64>> {
    settings.validate()?;
    if code.len() != settings.digits || !code.bytes().all(|b| b.is_ascii_digit()) {
        return Ok(None);
    }
    let totp = totp_with(secret, username, settings)?;
    for step in [
        at / settings.period,
        at / settings.period + 1,
        (at / settings.period).saturating_sub(1),
    ] {
        if last_step.is_none_or(|last| step > last)
            && constant_eq(&totp.generate(step * settings.period), code)
        {
            return Ok(Some(step));
        }
    }
    Ok(None)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SigningKey {
    #[serde(default)]
    pub remote: Option<crate::kms::RemoteKey>,
    #[serde(default = "default_signing_algorithm")]
    pub algorithm: String,
    pub kid: String,
    pub pem: String,
    pub created_at: u64,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct RetiredKey {
    pub jwk: Value,
    pub expires_at: u64,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Keys {
    pub active: SigningKey,
    pub retired: Vec<RetiredKey>,
}

fn default_signing_algorithm() -> String {
    "RS256".into()
}
impl SigningKey {
    pub fn generate() -> Result<Self> {
        Self::generate_algorithm("RS256")
    }
    pub fn generate_algorithm(algorithm: &str) -> Result<Self> {
        use aws_lc_rs::signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair, Ed25519KeyPair};
        let bytes = match algorithm {
            "RS256" => {
                let rsa = RsaKeyPair::generate(KeySize::Rsa3072).map_err(Error::internal)?;
                AsDer::<Pkcs8V1Der>::as_der(&rsa)
                    .map_err(Error::internal)?
                    .as_ref()
                    .to_vec()
            }
            "ES256" => EcdsaKeyPair::generate_pkcs8(
                &ECDSA_P256_SHA256_FIXED_SIGNING,
                &aws_lc_rs::rand::SystemRandom::new(),
            )
            .map_err(Error::internal)?
            .as_ref()
            .to_vec(),
            "EdDSA" => Ed25519KeyPair::generate_pkcs8(&aws_lc_rs::rand::SystemRandom::new())
                .map_err(Error::internal)?
                .as_ref()
                .to_vec(),
            _ => {
                return Err(Error::bad(
                    "Supported signing algorithms: RS256, ES256, EdDSA",
                ));
            }
        };
        Self::import(
            algorithm,
            &pem::encode(&pem::Pem::new("PRIVATE KEY", bytes)),
            None,
        )
    }
    pub fn import(algorithm: &str, value: &str, kid: Option<String>) -> Result<Self> {
        if value.len() > 16_384 {
            return Err(Error::bad("Private key exceeds 16 KiB"));
        }
        let parsed = pem::parse(value).map_err(|_| Error::bad("Invalid private PEM key"))?;
        let value = if algorithm == "RS256" && parsed.tag() == "RSA PRIVATE KEY" {
            let rsa = RsaKeyPair::from_der(parsed.contents())
                .map_err(|_| Error::bad("Invalid RSA private key"))?;
            let der = AsDer::<Pkcs8V1Der>::as_der(&rsa).map_err(Error::internal)?;
            pem::encode(&pem::Pem::new("PRIVATE KEY", der.as_ref()))
        } else {
            value.to_owned()
        };
        let key = Self {
            remote: None,
            algorithm: algorithm.into(),
            kid: kid.unwrap_or_else(id),
            pem: value,
            created_at: now(),
        };
        let jwk: crate::jose::PublicJwk =
            serde_json::from_value(key.jwk()?).map_err(Error::internal)?;
        jwk.validate()?;
        let proof =
            json!({"iss":"self-check","sub":"self-check","aud":"self-check","exp":now()+60});
        let signed = key.sign(&proof, false)?;
        crate::jose::PublicJwks { keys: vec![jwk] }.verify(&signed, "self-check", "self-check")?;
        Ok(key)
    }
    pub fn jwk(&self) -> Result<Value> {
        if let Some(remote) = &self.remote {
            remote.public_jwk.validate()?;
            if remote.public_jwk.kid != self.kid
                || remote.public_jwk.alg != self.algorithm
                || !self.pem.is_empty()
            {
                return Err(Error::bad("External key metadata mismatch"));
            }
            return serde_json::to_value(&remote.public_jwk).map_err(Error::internal);
        }
        use aws_lc_rs::signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair, Ed25519KeyPair};
        let pem = pem::parse(&self.pem).map_err(|_| Error::bad("Invalid private key PEM"))?;
        match self.algorithm.as_str() {
            "RS256" => {
                let key = RsaKeyPair::from_pkcs8(pem.contents())
                    .map_err(|_| Error::bad("Invalid RSA PKCS8 key"))?;
                let components = PublicKeyComponents::<Vec<u8>>::from(key.public_key());
                Ok(
                    json!({"kty":"RSA", "use":"sig", "alg":"RS256", "kid":self.kid,"n":URL_SAFE_NO_PAD.encode(components.n),"e":URL_SAFE_NO_PAD.encode(components.e)}),
                )
            }
            "ES256" => {
                let key = EcdsaKeyPair::from_private_key_der(
                    &ECDSA_P256_SHA256_FIXED_SIGNING,
                    pem.contents(),
                )
                .map_err(|_| Error::bad("Invalid P-256 key"))?;
                let point = key.public_key().as_ref();
                Ok(
                    json!({"kty":"EC","use":"sig","alg":"ES256","crv":"P-256","kid":self.kid,"x":URL_SAFE_NO_PAD.encode(&point[1..33]),"y":URL_SAFE_NO_PAD.encode(&point[33..65])}),
                )
            }
            "EdDSA" => {
                let key = Ed25519KeyPair::from_pkcs8(pem.contents())
                    .map_err(|_| Error::bad("Invalid Ed25519 key"))?;
                Ok(
                    json!({"kty":"OKP","use":"sig","alg":"EdDSA","crv":"Ed25519","kid":self.kid,"x":URL_SAFE_NO_PAD.encode(key.public_key().as_ref())}),
                )
            }
            _ => Err(Error::bad("Unsupported signing algorithm")),
        }
    }
    pub fn sign(&self, claims: &Value, access: bool) -> Result<String> {
        self.sign_type(claims, if access { "at+jwt" } else { "JWT" })
    }
    pub fn sign_type(&self, claims: &Value, typ: &str) -> Result<String> {
        if self.remote.is_some() {
            return Err(Error::bad("External key requires its configured signer"));
        }
        let (algorithm, encoding) = match self.algorithm.as_str() {
            "RS256" => (
                Algorithm::RS256,
                EncodingKey::from_rsa_pem(self.pem.as_bytes()),
            ),
            "ES256" => (
                Algorithm::ES256,
                EncodingKey::from_ec_pem(self.pem.as_bytes()),
            ),
            "EdDSA" => (
                Algorithm::EdDSA,
                EncodingKey::from_ed_pem(self.pem.as_bytes()),
            ),
            _ => return Err(Error::bad("Unsupported signing algorithm")),
        };
        let mut header = Header::new(algorithm);
        header.kid = Some(self.kid.clone());
        header.typ = Some(typ.into());
        jsonwebtoken::encode(&header, claims, &encoding.map_err(Error::internal)?)
            .map_err(Error::internal)
    }
}
