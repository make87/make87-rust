use super::{EncodeError, Encoder};
use serde::ser::{Serialize};
use serde::de::DeserializeOwned;
use serde_yaml::{from_slice, to_string};

pub struct YamlEncoder;

impl<T> Encoder<T> for YamlEncoder
where
    T: Serialize + DeserializeOwned,
{
    fn encode(&self, value: &T) -> Result<Vec<u8>, EncodeError> {
        to_string(value)
            .map(|s| s.into_bytes())
            .map_err(|e| EncodeError(e.to_string()))
    }

    fn decode(&self, data: &[u8]) -> Result<T, EncodeError> {
        from_slice(data).map_err(|e| EncodeError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::Deserializer;
    use serde::{Deserialize, Serialize, Serializer};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Example {
        id: u32,
        name: String,
    }

    #[test]
    fn test_yaml_encoder_roundtrip() {
        let encoder = YamlEncoder;
        let original = Example { id: 42, name: "hello".to_string() };

        // Encode to YAML bytes
        let yaml_bytes = encoder.encode(&original).expect("encode failed");
        let yaml_str = std::str::from_utf8(&yaml_bytes).unwrap();
        assert!(yaml_str.contains("id: 42"));
        assert!(yaml_str.contains("name: hello"));

        // Decode back
        let decoded: Example = encoder.decode(&yaml_bytes).expect("decode failed");
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_yaml_encoder_decode_error() {
        let encoder = YamlEncoder;
        // Not valid YAML
        let bad_yaml = b"{ not yaml: }";
        let result: Result<Example, _> = encoder.decode(bad_yaml);
        assert!(result.is_err());
    }

    struct NotSerializable;

    impl Serialize for NotSerializable {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(serde::ser::Error::custom("not serializable"))
        }
    }

    impl<'de> Deserialize<'de> for NotSerializable {
        fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Ok(NotSerializable)
        }
    }

    #[test]
    fn test_yaml_encoder_encode_error() {
        let encoder = YamlEncoder;
        let value = NotSerializable;
        let result = encoder.encode(&value);
        assert!(result.is_err());
    }
}
