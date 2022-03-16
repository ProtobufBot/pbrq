pub trait PbToBytes<B>
where
    B: prost::Message,
{
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(buf: &[u8]) -> Result<B, prost::DecodeError>;
}

impl<B> PbToBytes<B> for B
where
    B: prost::Message + Default,
{
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        prost::Message::encode(self, &mut buf).expect("prost encode failed");
        buf
    }
    fn from_bytes(buf: &[u8]) -> Result<Self, prost::DecodeError> {
        prost::Message::decode(buf)
    }
}
