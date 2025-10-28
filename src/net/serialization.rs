/// Serialization utilities for network messages
///
/// Provides helper functions for serializing and deserializing messages
/// using bincode (fast binary format).

use serde::{Serialize, Deserialize};
use anyhow::Result;

/// Serialize a message to bytes using bincode
pub fn serialize<T: Serialize>(message: &T) -> Result<Vec<u8>> {
    Ok(bincode::serialize(message)?)
}

/// Deserialize bytes to a message using bincode
pub fn deserialize<'a, T: Deserialize<'a>>(bytes: &'a [u8]) -> Result<T> {
    Ok(bincode::deserialize(bytes)?)
}

/// Serialize with size prefix (4 bytes)
pub fn serialize_with_length<T: Serialize>(message: &T) -> Result<Vec<u8>> {
    let data = bincode::serialize(message)?;
    let len = data.len() as u32;

    let mut result = Vec::with_capacity(4 + data.len());
    result.extend_from_slice(&len.to_le_bytes());
    result.extend_from_slice(&data);

    Ok(result)
}

/// Deserialize with size prefix
pub fn deserialize_with_length<'a, T: Deserialize<'a>>(bytes: &'a [u8]) -> Result<T> {
    if bytes.len() < 4 {
        anyhow::bail!("Buffer too small for length prefix");
    }

    let len = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;

    if bytes.len() < 4 + len {
        anyhow::bail!("Buffer too small for message");
    }

    Ok(bincode::deserialize(&bytes[4..4 + len])?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestMessage {
        id: u32,
        data: String,
    }

    #[test]
    fn test_serialize_deserialize() {
        let msg = TestMessage {
            id: 42,
            data: "Hello, World!".to_string(),
        };

        let bytes = serialize(&msg).unwrap();
        let decoded: TestMessage = deserialize(&bytes).unwrap();

        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_serialize_with_length() {
        let msg = TestMessage {
            id: 123,
            data: "Test".to_string(),
        };

        let bytes = serialize_with_length(&msg).unwrap();
        let decoded: TestMessage = deserialize_with_length(&bytes).unwrap();

        assert_eq!(msg, decoded);
    }
}
