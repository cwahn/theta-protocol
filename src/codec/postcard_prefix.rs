use bytes::BytesMut;
use core::marker::PhantomData;
use serde::{Deserialize, Serialize};
use std::io;
use tokio_util::codec::{Decoder, Encoder};

/// A minimal codec using postcard serialization with u32 length prefixes.
///
/// Length prefix encoding stores the length of each message as a 4-byte u32 at the beginning,
/// making it efficient for framing with fixed-size length headers.
#[derive(Debug, Clone)]
pub struct PostcardPrefixCodec<T>(PhantomData<T>);

impl<T> Default for PostcardPrefixCodec<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: Serialize> Encoder<T> for PostcardPrefixCodec<T> {
    type Error = io::Error;

    fn encode(&mut self, item: T, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let encoded = postcard::to_stdvec(&item)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Encode length as u32 in little-endian format
        let length = encoded.len() as u32;
        let length_bytes = length.to_le_bytes();

        dst.reserve(4 + encoded.len());
        dst.extend_from_slice(&length_bytes);
        dst.extend_from_slice(&encoded);
        Ok(())
    }
}

impl<T: for<'de> Deserialize<'de>> Decoder for PostcardPrefixCodec<T> {
    type Item = T;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }

        // Read the u32 length prefix in little-endian format
        let length_bytes: [u8; 4] = src[0..4]
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid length prefix"))?;
        let length = u32::from_le_bytes(length_bytes) as usize;

        // Check if we have enough bytes for the complete message
        if src.len() < 4 + length {
            return Ok(None);
        }

        // Extract the frame (skip length prefix)
        let frame = src.split_to(4 + length);
        let message_bytes = &frame[4..];

        // Decode the message
        postcard::from_bytes(message_bytes)
            .map(Some)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        string::{String, ToString},
        vec,
        vec::Vec,
    };

    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    struct TestMessage {
        id: u32,
        data: Vec<u8>,
        text: String,
    }

    #[tokio::test]
    async fn test_roundtrip() {
        let mut codec = PostcardPrefixCodec::<TestMessage>::default();

        let message = TestMessage {
            id: 42,
            data: vec![0, 1, 0, 2, 0, 3], // Test with zeros (should work fine)
            text: "Hello\0World".to_string(),
        };

        let mut buffer = BytesMut::new();
        codec.encode(message.clone(), &mut buffer).unwrap();

        // Verify that zeros are preserved (unlike COBS)
        assert!(buffer.iter().any(|&b| b == 0));

        let decoded = codec.decode(&mut buffer).unwrap().unwrap();
        assert_eq!(message, decoded);
        assert!(buffer.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_messages() {
        let mut codec = PostcardPrefixCodec::<TestMessage>::default();

        let messages = vec![
            TestMessage {
                id: 1,
                data: vec![0, 1],
                text: "First".to_string(),
            },
            TestMessage {
                id: 2,
                data: vec![2, 0],
                text: "Second".to_string(),
            },
            TestMessage {
                id: 3,
                data: vec![0, 0],
                text: "Third".to_string(),
            },
        ];

        let mut buffer = BytesMut::new();
        for msg in &messages {
            codec.encode(msg.clone(), &mut buffer).unwrap();
        }

        let mut decoded = Vec::new();
        while let Some(msg) = codec.decode(&mut buffer).unwrap() {
            decoded.push(msg);
        }

        assert_eq!(messages, decoded);
    }

    #[tokio::test]
    async fn test_partial_frame() {
        let mut codec = PostcardPrefixCodec::<TestMessage>::default();

        let message = TestMessage {
            id: 99,
            data: vec![1, 2, 3],
            text: "Test".to_string(),
        };

        // Encode full message
        let mut full_buffer = BytesMut::new();
        codec.encode(message.clone(), &mut full_buffer).unwrap();

        // Test partial decode (only length prefix)
        let mut partial = BytesMut::new();
        partial.extend_from_slice(&full_buffer[..1]); // Only first byte

        assert!(codec.decode(&mut partial).unwrap().is_none());

        // Add more bytes but still incomplete
        partial.extend_from_slice(&full_buffer[1..full_buffer.len() - 1]);
        assert!(codec.decode(&mut partial).unwrap().is_none());

        // Add final byte
        partial.extend_from_slice(&[full_buffer[full_buffer.len() - 1]]);
        let decoded = codec.decode(&mut partial).unwrap().unwrap();
        assert_eq!(message, decoded);
    }

    #[tokio::test]
    async fn test_empty_data() {
        let mut codec = PostcardPrefixCodec::<TestMessage>::default();

        let message = TestMessage {
            id: 0,
            data: vec![],
            text: String::new(),
        };

        let mut buffer = BytesMut::new();
        codec.encode(message.clone(), &mut buffer).unwrap();

        let decoded = codec.decode(&mut buffer).unwrap().unwrap();
        assert_eq!(message, decoded);
    }
}
