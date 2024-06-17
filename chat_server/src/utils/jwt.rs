use crate::{AppError, User};
use jwt_simple::prelude::*;

const JWT_DURATION: u64 = 60 * 60 * 24 * 7; // 1 week
const JWT_ISSUER: &str = "chat_server";
const JWT_AUDIENCE: &str = "chat_web";

pub struct EncodingKey(Ed25519KeyPair);

pub struct DecodingKey(Ed25519PublicKey);

impl EncodingKey {
    pub fn load(pem: &str) -> Result<Self, AppError> {
        Ok(Self(Ed25519KeyPair::from_pem(pem)?))
    }

    pub fn sign(&self, user: impl Into<User>) -> Result<String, AppError> {
        let claims = Claims::with_custom_claims(user.into(), Duration::from_secs(JWT_DURATION));
        let claims = claims.with_issuer(JWT_ISSUER).with_audience(JWT_AUDIENCE);
        Ok(self.0.sign(claims)?)
    }
}

impl DecodingKey {
    pub fn load(pem: &str) -> Result<Self, AppError> {
        Ok(Self(Ed25519PublicKey::from_pem(pem)?))
    }

    #[allow(unused)]
    pub fn verify(&self, token: &str) -> Result<User, AppError> {
        let options = VerificationOptions {
            allowed_issuers: Some(HashSet::from_strings(&[JWT_ISSUER])),
            allowed_audiences: Some(HashSet::from_strings(&[JWT_AUDIENCE])),
            ..Default::default()
        };

        let claims = self.0.verify_token::<User>(token, Some(options))?;
        Ok(claims.custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt() -> anyhow::Result<()> {
        let encoding_pem = include_str!("../../fixtures/encoding.pem");
        let decoding_pem = include_str!("../../fixtures/decoding.pem");

        let ek = EncodingKey::load(encoding_pem)?;
        let dk = DecodingKey::load(decoding_pem)?;

        let user = User::new(123, "my_name", "test@abc.com");

        let token = ek.sign(user.clone())?;
        let user2 = dk.verify(&token)?;

        assert_eq!(user, user2);

        Ok(())
    }
}
