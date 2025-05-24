use prost::Message;
use std::marker::PhantomData;
use super::{Encoder, EncodeError};

pub struct ProtobufEncoder<T> {
    _marker: PhantomData<T>,
}

impl<T> ProtobufEncoder<T> {
    pub fn new() -> Self {
        ProtobufEncoder { _marker: PhantomData }
    }
}

impl<T> Encoder<T> for ProtobufEncoder<T>
where
    T: Message + Default,
{
    fn encode(&self, value: &T) -> Result<Vec<u8>, EncodeError> {
        let mut buf = Vec::with_capacity(value.encoded_len());
        value.encode(&mut buf).map_err(|e| EncodeError(e.to_string()))?;
        Ok(buf)
    }

    fn decode(&self, data: &[u8]) -> Result<T, EncodeError> {
        T::decode(data).map_err(|e| EncodeError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost::{bytes, Message};

    #[derive(Clone, PartialEq, Message)]
    pub struct Example {
        #[prost(uint32, tag = "1")]
        pub id: u32,
        #[prost(string, tag = "2")]
        pub name: String,
    }

    #[test]
    fn test_protobuf_encoder_roundtrip() {
        let encoder = ProtobufEncoder::<Example>::new();
        let original = Example { id: 42, name: "hello".to_string() };
        let encoded = encoder.encode(&original).expect("encode failed");
        let decoded: Example = encoder.decode(&encoded).expect("decode failed");
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_protobuf_encoder_decode_error() {
        let encoder = ProtobufEncoder::<Example>::new();
        let bad_data = b"not protobuf";
        let result: Result<Example, _> = encoder.decode(bad_data);
        assert!(result.is_err());
    }

    #[derive(Default, Debug, PartialEq)]
    struct NotSerializable;
    impl Message for NotSerializable {
        fn encode_raw(&self, _buf: &mut impl bytes::BufMut) {}
        fn merge_field(&mut self, _tag: u32, _wire_type: prost::encoding::WireType, _buf: &mut impl bytes::Buf, _ctx: prost::encoding::DecodeContext) -> Result<(), prost::DecodeError> {
            Ok(())
        }
        fn encoded_len(&self) -> usize { 0 }
        fn clear(&mut self) {}
    }

    #[test]
    fn test_protobuf_encoder_encode_error() {
        let encoder = ProtobufEncoder::<FailingMessage>::new();
        #[derive(Debug, Default)]
        struct FailingMessage;
        impl Message for FailingMessage {
            fn encode_raw(&self, _buf: &mut impl bytes::BufMut) {
                panic!("encode error");
            }
            fn merge_field(&mut self, _tag: u32, _wire_type: prost::encoding::WireType, _buf: &mut impl bytes::Buf, _ctx: prost::encoding::DecodeContext) -> Result<(), prost::DecodeError> {
                Ok(())
            }
            fn encoded_len(&self) -> usize { 1 }
            fn clear(&mut self) {}
        }
        let value = FailingMessage;
        let result = std::panic::catch_unwind(|| encoder.encode(&value));
        assert!(result.is_err());
    }
}
