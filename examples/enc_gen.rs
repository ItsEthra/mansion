use chacha20poly1305::aead::rand_core::OsRng;
use rsa::{RsaPrivateKey, pkcs8::ToPrivateKey, pkcs1::ToRsaPublicKey};

fn main() {
    let private = RsaPrivateKey::new(&mut OsRng, 1024).unwrap();
    let public = private.to_public_key();

    let doc = private.to_pkcs8_pem().unwrap();
    std::fs::write("private.pem", doc.as_bytes()).unwrap();

    let doc = public.to_pkcs1_pem().unwrap();
    std::fs::write("public.pem", doc.as_bytes()).unwrap();
}
