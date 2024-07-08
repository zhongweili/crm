use jwt_simple::prelude::*;
use tonic::{service::Interceptor, Request, Status};
use tracing::info;

pub struct EncodingKey(Ed25519KeyPair);

#[derive(Debug, Clone)]
pub struct DecodingKey(Ed25519PublicKey);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: i64,
    pub fullname: String,
    pub email: String,
}

const JWT_DURATION: u64 = 60 * 60 * 24 * 7;
const JWT_ISS: &str = "crm";
const JWT_AUD: &str = "crm_client";

impl EncodingKey {
    pub fn load(pem: &str) -> Result<Self, jwt_simple::Error> {
        Ok(Self(Ed25519KeyPair::from_pem(pem)?))
    }

    pub fn sign(&self, user: impl Into<User>) -> Result<String, jwt_simple::Error> {
        let claims = Claims::with_custom_claims(user.into(), Duration::from_secs(JWT_DURATION));
        let claims = claims.with_issuer(JWT_ISS).with_audience(JWT_AUD);
        self.0.sign(claims)
    }
}

impl DecodingKey {
    pub fn load(pem: &str) -> Result<Self, jwt_simple::Error> {
        Ok(Self(Ed25519PublicKey::from_pem(pem)?))
    }

    #[allow(unused)]
    pub fn verify(&self, token: &str) -> Result<User, jwt_simple::Error> {
        let opts = VerificationOptions {
            allowed_issuers: Some(HashSet::from_strings(&[JWT_ISS])),
            allowed_audiences: Some(HashSet::from_strings(&[JWT_AUD])),
            ..Default::default()
        };

        let claims = self.0.verify_token::<User>(token, Some(opts))?;
        Ok(claims.custom)
    }
}

impl Interceptor for DecodingKey {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        let token = req
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok());
        info!("token: {:?}", token);
        let user = match token {
            Some(bearer) => {
                let token = bearer
                    .strip_prefix("Bearer ")
                    .ok_or_else(|| Status::unauthenticated("invalid token format"))?;
                self.verify(token)
                    .map_err(|e| Status::unauthenticated(e.to_string()))?
            }
            None => return Err(Status::unauthenticated("missing token")),
        };

        req.extensions_mut().insert(user);
        Ok(req)
    }
}

impl User {
    pub fn new(id: i64, fullname: String, email: String) -> Self {
        Self {
            id,
            fullname,
            email,
        }
    }
}
