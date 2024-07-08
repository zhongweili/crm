use std::fs::File;
use std::io::Write;
use std::path::Path;

use anyhow::Result;
use crm::abi::auth::{EncodingKey, User};
use crm::AppConfig;

fn main() -> Result<()> {
    let encoding_pem = AppConfig::load()?.auth.ek;
    let ek = EncodingKey::load(&encoding_pem)?;

    let user = User::new(1, "Tyr Chen".to_string(), "tchen@acme.org".to_string());

    let token = ek.sign(user.clone())?;

    let token_path = "../fixtures/token";
    let mut file = File::create(Path::new(token_path))?;
    file.write_all(token.as_bytes())?;

    println!("Generated token: {}", token);
    Ok(())
}
