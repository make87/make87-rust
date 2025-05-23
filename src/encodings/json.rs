use serde::{Serialize, de::DeserializeOwned};
use super::{Encoder, EncodeError};

pub struct JsonEncoder;

impl<T> Encoder<T> for JsonEncoder
where
    T: Serialize + DeserializeOwned,
{
    fn encode(&self, value: &T) -> Result<Vec<u8>, EncodeError> {
        serde_json::to_vec(value).map_err(|e| EncodeError(e.to_string()))
    }

    fn decode(&self, data: &[u8]) -> Result<T, EncodeError> {
        serde_json::from_slice(data).map_err(|e| EncodeError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Deserialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Example {
        id: u32,
        name: String,
    }

    #[test]
    fn test_json_encoder_roundtrip() {
        let encoder = JsonEncoder;
        let original = Example { id: 42, name: "hello".to_string() };
        // Encode to JSON bytes
        let json_bytes = encoder.encode(&original).expect("encode failed");
        let json_str = std::str::from_utf8(&json_bytes).unwrap();
        assert!(json_str.contains("\"id\":42"));
        assert!(json_str.contains("\"name\":\"hello\""));
        // Decode back
        let decoded: Example = encoder.decode(&json_bytes).expect("decode failed");
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_json_encoder_decode_error() {
        let encoder = JsonEncoder;
        // Not valid JSON
        let bad_json = b"{ not json: }";
        let result: Result<Example, _> = encoder.decode(bad_json);
        assert!(result.is_err());
    }

    struct NotSerializable;
    impl Serialize for NotSerializable {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Err(serde::ser::Error::custom("not serializable"))
        }
    }
    impl<'de> Deserialize<'de> for NotSerializable {
        fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            Ok(NotSerializable)
        }
    }

    #[test]
    fn test_json_encoder_encode_error() {
        let encoder = JsonEncoder;
        let value = NotSerializable;
        let result = encoder.encode(&value);
        assert!(result.is_err());
    }
}
