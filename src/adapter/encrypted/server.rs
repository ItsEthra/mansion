use crate::{MessageType, EncryptionTarget, Adapter, Error};
use chacha20poly1305::{aead::{self, NewAead, Aead}, ChaCha20Poly1305, Nonce};
use x25519_dalek::{EphemeralSecret, PublicKey};
use rsa::{RsaPrivateKey, PaddingScheme};
use rand::{rngs::OsRng, RngCore};
use std::marker::PhantomData;
use tokio::sync::OnceCell;
use cursored::Cursored;

pub struct EncryptedServerAdapter<M: MessageType + EncryptionTarget> {
    shared: OnceCell<ChaCha20Poly1305>,
    private_rsa: RsaPrivateKey,
    secret: EphemeralSecret,
    _ph: PhantomData<M>
}

impl<M: MessageType + EncryptionTarget> EncryptedServerAdapter<M> {
    pub fn new(server_private_rsa: RsaPrivateKey) -> Self {
        Self {
            private_rsa: server_private_rsa,
            secret: EphemeralSecret::new(aead::rand_core::OsRng),
            shared: OnceCell::new(),
            _ph: PhantomData,
        }
    }
}

impl<M: MessageType + EncryptionTarget> Adapter for EncryptedServerAdapter<M> {
    type Message = M;

    fn cloned(&self) -> Box<dyn Adapter<Message = Self::Message>> {
        Box::new(Self {
            secret: EphemeralSecret::new(aead::rand_core::OsRng),
            private_rsa: self.private_rsa.clone(),
            shared: OnceCell::new(),
            _ph: PhantomData,
        })
    }

    fn encode(&self, msg: Self::Message, buf: &mut Cursored) -> Result<(), crate::Error> {
        if msg.is_response() {
            buf.put_u8(0);
            buf.put_slice(PublicKey::from(&self.secret).as_bytes());
            Ok(())
        } else if let Some(key) = self.shared.get() {
            let mut nonce = [0; 12];
            OsRng.fill_bytes(&mut nonce[..]);

            let data = bincode::serialize(&msg)?;

            let ciphertext = key.encrypt(Nonce::from_slice(&nonce[..]), &data[..])
                .map_err(|e| Error::custom(e))?;

            buf.put_u8(0);
            buf.put_slice(&nonce[..]);
            buf.put_slice(&ciphertext[..]);

            Ok(())
        } else {
            unreachable!()
        }
    }

    fn decode(&self, buf: &mut Cursored) -> Result<Self::Message, crate::Error> {
        let enc = buf.get_u8();
        if enc == 0 {
            let ciphertext = buf.lasting();
            let ecdh: [u8; 32] = self.private_rsa.decrypt(PaddingScheme::PKCS1v15Encrypt, ciphertext)
                .map_err(|e| Error::custom(e))?
                .try_into()
                .unwrap();

            let shared = unsafe { std::ptr::read(&self.secret as *const EphemeralSecret) }
                    .diffie_hellman(&PublicKey::from(ecdh));
            let _ = self.shared.set(ChaCha20Poly1305::new_from_slice(shared.as_bytes()).unwrap());

            Ok(<Self::Message as EncryptionTarget>::request())
        } else if enc == 1 {
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
        } else {
            unreachable!()
        }
    }
}