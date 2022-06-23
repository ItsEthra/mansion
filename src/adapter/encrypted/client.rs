use chacha20poly1305::{
    ChaCha20Poly1305,
    aead::{Aead, NewAead, self},
    Nonce,
};
use rsa::{RsaPublicKey, PublicKey as AeadPublicKey, PaddingScheme};
use x25519_dalek::{EphemeralSecret, PublicKey};
use crate::{Adapter, MessageType, Error};
use rand::{rngs::OsRng, RngCore};
use std::marker::PhantomData;
use super::EncryptionTarget;
use tokio::sync::OnceCell;
use cursored::Cursored;

pub struct EncryptedClientAdapter<M: MessageType + EncryptionTarget> {
    shared: OnceCell<ChaCha20Poly1305>,
    secret: EphemeralSecret,
    server: RsaPublicKey,
    _ph: PhantomData<M>,
}

impl<M: MessageType + EncryptionTarget> EncryptedClientAdapter<M> {
    pub fn new(server_public_key: RsaPublicKey) -> Self {
        Self {
            secret: EphemeralSecret::new(aead::rand_core::OsRng),
            server: server_public_key,
            shared: OnceCell::new(),
            _ph: PhantomData,
        }
    }
}

impl<M: MessageType + EncryptionTarget> Adapter for EncryptedClientAdapter<M> {
    type Message = M;

    fn encode(&self, msg: Self::Message, buf: &mut Cursored) -> Result<(), crate::Error> {
        if msg.is_request() {
            log::trace!("Encoding request for encryption");

            let public = PublicKey::from(&self.secret);
            log::trace!("Public key: {:?}", public.as_bytes());

            let ciphertext = self
                .server
                .encrypt(
                    &mut aead::rand_core::OsRng,
                    PaddingScheme::PKCS1v15Encrypt,
                    public.as_bytes(),
                )
                .map_err(|e| Error::custom(e))?;
            log::trace!("Ciphertext: {ciphertext:?}");

            buf.put_u8(0);
            buf.put_slice(&ciphertext[..]);
        } else if let Some(key) = self.shared.get() {
            log::trace!("Encoding a message when encryption is established");

            let mut nonce = [0; 12];
            OsRng.fill_bytes(&mut nonce);
            log::trace!("Nonce: {nonce:?}");

            let data = bincode::serialize(&msg)?;
            log::trace!("Data: {data:?}");

            let ciphertext = key
                .encrypt(Nonce::from_slice(&nonce), &data[..])
                .map_err(|e| Error::custom(e))?;
            log::trace!("Ciphertext: {ciphertext:?}");

            buf.put_u8(1);
            buf.put_slice(&nonce);
            buf.put_slice(&ciphertext);
        }

        Ok(())
    }

    fn decode(&self, buf: &mut Cursored) -> Result<Self::Message, crate::Error> {
        let enc = buf.get_u8();
        if enc == 0 {
            log::trace!("Decoding a message when encryption is not yet established. Supposing response");

            let ecdh: [u8; 32] = buf.get_slice(32).try_into().unwrap();
            log::trace!("Ecdh: {ecdh:?}");

            let shared = unsafe { std::ptr::read(&self.secret as *const EphemeralSecret) }
                .diffie_hellman(&PublicKey::from(ecdh));
            let _ = self
                .shared
                .set(ChaCha20Poly1305::new_from_slice(shared.as_bytes()).unwrap());

            Ok(<Self::Message as EncryptionTarget>::response())
        } else if let Some(key) = self.shared.get() {
            log::trace!("Decoding a message when encryption is established");

            let nonce: [u8; 12] = buf.get_slice(12).try_into().unwrap();
            log::trace!("Nonce: {nonce:?}");

            let ciphertext = buf.lasting();
            log::trace!("Ciphertext: {ciphertext:?}");

            let data = key
                .decrypt(Nonce::from_slice(&nonce), ciphertext)
                .map_err(|e| Error::custom(e))?;
            log::trace!("Data: {data:?}");

            let msg = bincode::deserialize(&data[..])?;
            Ok(msg)
        } else {
            unreachable!()
        }
    }

    #[cfg(feature = "server")]
    fn cloned(&self) -> Box<dyn Adapter<Message = Self::Message>> {
        todo!()
    }
}
