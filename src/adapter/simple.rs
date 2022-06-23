use crate::{MessageType, Adapter};
use std::marker::PhantomData;
use cursored::Cursored;

pub struct SimpleAdapter<M: MessageType>(PhantomData<M>);
impl<M: MessageType> Default for SimpleAdapter<M> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<M: MessageType> Adapter for SimpleAdapter<M> {
    type Message = M;

    fn cloned(&self) -> Box<dyn Adapter<Message = Self::Message>> {
        Box::new(SimpleAdapter(PhantomData))
    }

    fn encode(&self, msg: Self::Message, buf: &mut Cursored) -> Result<(), crate::Error> {
        let data = bincode::serialize(&msg)?;
        buf.put_slice(&data[..]);

        Ok(())
    }

    fn decode(&self, buf: &mut Cursored) -> Result<Self::Message, crate::Error> {
        Ok(bincode::deserialize::<M>(buf.lasting())?)
    }
}