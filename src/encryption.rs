//! Compact nested JWT encryption: RSA-OAEP-256 and A256GCM (RFC 7516).
use crate::{
    crypto,
    error::{Error, Result},
};
use aws_lc_rs::rsa::{OAEP_SHA256_MGF1SHA256, OaepPublicEncryptingKey, PublicEncryptingKey};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::{TryRng, rngs::SysRng};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EncryptionKey {
    #[serde(default = "default_content_encryption")]
    pub content_encryption: String,
    pub kid: String,
    pub public_key_pem: String,
}
fn default_content_encryption() -> String {
    "A256GCM".into()
}
impl EncryptionKey {
    fn key(&self) -> Result<PublicEncryptingKey> {
        if self.kid.is_empty()
            || self.kid.len() > 128
            || !self.kid.bytes().all(|b| b.is_ascii_graphic())
            || self.public_key_pem.len() > 8192
        {
            return Err(Error::bad("Invalid encryption key identifier or size"));
        }
        let parsed = pem::parse(&self.public_key_pem)
            .map_err(|_| Error::bad("Invalid public encryption PEM"))?;
        if parsed.tag() != "PUBLIC KEY" {
            return Err(Error::bad(
                "Encryption requires an X.509 public key, never a private key",
            ));
        }
        let key = PublicEncryptingKey::from_der(parsed.contents())
            .map_err(|_| Error::bad("Invalid RSA encryption key"))?;
        if !(2048..=4096).contains(&key.key_size_bits()) {
            return Err(Error::bad("RSA encryption key must be 2048–4096 bits"));
        }
        Ok(key)
    }
    pub fn validate(&self) -> Result<()> {
        if !["A256GCM", "A256CBC-HS512"].contains(&self.content_encryption.as_str()) {
            return Err(Error::bad(
                "Supported content encryption: A256GCM or A256CBC-HS512",
            ));
        }
        self.key().map(|_| ())
    }
    pub fn encrypt(&self, signed_jwt: &str) -> Result<String> {
        self.validate()?;
        let key = OaepPublicEncryptingKey::new(self.key()?).map_err(Error::internal)?;
        let mut cek = zeroize::Zeroizing::new(vec![
            0u8;
            if self.content_encryption == "A256GCM" {
                32
            } else {
                64
            }
        ]);
        SysRng.try_fill_bytes(&mut cek).map_err(Error::internal)?;
        let protected = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&json!({"alg":"RSA-OAEP-256","enc":self.content_encryption,"cty":"JWT","kid":self.kid})).map_err(Error::internal)?);
        let mut wrapped = vec![0; key.ciphertext_size()];
        let wrapped = key
            .encrypt(&OAEP_SHA256_MGF1SHA256, &cek, &mut wrapped, None)
            .map_err(Error::internal)?;
        let (iv, ciphertext, tag) = if self.content_encryption == "A256GCM" {
            let cek: &[u8; 32] = cek.as_slice().try_into().map_err(Error::internal)?;
            let sealed = crypto::seal(cek, protected.as_bytes(), signed_jwt.as_bytes())?;
            let body = sealed
                .strip_prefix(b"RIAUTH-AEAD1")
                .ok_or_else(|| Error::internal("Invalid local AEAD envelope"))?;
            (
                body[..12].to_vec(),
                body[12..body.len() - 16].to_vec(),
                body[body.len() - 16..].to_vec(),
            )
        } else {
            use aws_lc_rs::{
                cipher::{AES_256, PaddedBlockEncryptingKey, UnboundCipherKey},
                hmac,
            };
            let key = PaddedBlockEncryptingKey::cbc_pkcs7(
                UnboundCipherKey::new(&AES_256, &cek[32..]).map_err(Error::internal)?,
            )
            .map_err(Error::internal)?;
            let mut ciphertext = signed_jwt.as_bytes().to_vec();
            let context = key.encrypt(&mut ciphertext).map_err(Error::internal)?;
            let iv: &[u8] = (&context).try_into().map_err(Error::internal)?;
            let mut mac = hmac::Context::with_key(&hmac::Key::new(hmac::HMAC_SHA512, &cek[..32]));
            mac.update(protected.as_bytes());
            mac.update(iv);
            mac.update(&ciphertext);
            mac.update(&((protected.len() as u64) * 8).to_be_bytes());
            (iv.to_vec(), ciphertext, mac.sign().as_ref()[..32].to_vec())
        };
        Ok(format!(
            "{}.{}.{}.{}.{}",
            protected,
            URL_SAFE_NO_PAD.encode(wrapped),
            URL_SAFE_NO_PAD.encode(iv),
            URL_SAFE_NO_PAD.encode(ciphertext),
            URL_SAFE_NO_PAD.encode(tag)
        ))
    }
}
