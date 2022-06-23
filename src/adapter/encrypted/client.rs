use chacha20poly1305::{ChaCha20Poly1305, aead::{Aead, NewAead, self}, Nonce};
use x25519_dalek::{EphemeralSecret, PublicKey as XPublicKey};
use rsa::{RsaPublicKey, PublicKey, PaddingScheme};
use crate::{Adapter, MessageType, Error};
use rand::{rngs::OsRng, RngCore};
use std::marker::PhantomData;
use super::EncryptionTarget;
use tokio::sync::OnceCell;
use cursored::Cursored;

pub struct EncryptedClientAdapter<M: MessageType + EncryptionTarget> {
    secret: OnceCell<EphemeralSecret>,
    shared: OnceCell<ChaCha20Poly1305>,
    server: RsaPublicKey,
    _ph: PhantomData<M>,
}

impl<M: MessageType + EncryptionTarget> EncryptedClientAdapter<M> {
    pub fn new(server_public_key: RsaPublicKey) -> Self {
        Self {
            server: server_public_key,
            secret: OnceCell::new(),
            shared: OnceCell::new(),
            _ph: PhantomData,
        }
    }
}

impl<M: MessageType + EncryptionTarget> Adapter for EncryptedClientAdapter<M> {
    type Message = M;

    fn encode(&self, msg: Self::Message, buf: &mut Cursored) -> Result<(), crate::Error> {
        if msg.request() {
            let secret = EphemeralSecret::new(aead::rand_core::OsRng);
            let public = XPublicKey::from(&secret);
            
            let ciphertext = self.server.encrypt(
                &mut aead::rand_core::OsRng,
                PaddingScheme::PKCS1v15Encrypt,
                public.as_bytes()
            ).map_err(|e| Error::custom(e))?;

            buf.put_u8(0);
            buf.put_slice(&ciphertext[..]);

        } else if let Some(key) = self.shared.get() {
            let mut nonce = [0; 12];
            OsRng.fill_bytes(&mut nonce);

            let data = bincode::serialize(&msg)?;
            let ciphertext = key.encrypt(Nonce::from_slice(&nonce), &data[..])
                .map_err(|e| Error::custom(e))?;

            buf.put_u8(1);
            buf.put_slice(&nonce);
            buf.put_slice(&ciphertext);
        }

        Ok(())
    }

    fn decode(&self, buf: &mut Cursored) -> Result<Self::Message, crate::Error> {
        let enc = buf.get_u8();
        if enc == 1 {
            if let Some(key) = self.shared.get() {
                let nonce: [u8; 12] = buf.get_slice(12).try_into().unwrap();
                let ciphertext = buf.lasting();

                let data = key.decrypt(Nonce::from_slice(&nonce), ciphertext)
                    .map_err(|e| Error::custom(e))?;
                
                let msg = bincode::deserialize(&data[..])?;
                Ok(msg)
            } else {
                unreachable!()
            }
        } else if enc == 0 {
            if let Some(sk) = self.secret.get() {
                let ecdh: [u8; 32] = buf.get_slice(32).try_into().unwrap();
                let shared = unsafe { std::ptr::read(sk as *const EphemeralSecret) }
                    .diffie_hellman(&XPublicKey::from(ecdh));
                let _ = self.shared.set(ChaCha20Poly1305::new_from_slice(shared.as_bytes()).unwrap());

                Ok(<Self::Message as EncryptionTarget>::response())
            } else {
                unreachable!()
            }
        } else {
            unreachable!()
        }
    }

    #[cfg(feature = "server")]
    fn cloned(&self) -> Box<dyn Adapter<Message = Self::Message>> {
        todo!()
    }
}