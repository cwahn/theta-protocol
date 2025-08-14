use bytes::BytesMut;
use core::marker::PhantomData;
use serde::{Deserialize, Serialize};
use std::io;
use tokio_util::codec::{Decoder, Encoder};

/// A minimal codec using postcard serialization with length prefixes.
///
/// Length prefix encoding stores the length of each message at the beginning,
/// making it efficient for framing but requiring knowledge of the full message size.
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

        // Encode length as varint (postcard uses LEB128)
        let length_bytes = postcard::to_stdvec(&encoded.len())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        dst.reserve(length_bytes.len() + encoded.len());
        dst.extend_from_slice(&length_bytes);
        dst.extend_from_slice(&encoded);
        Ok(())
    }
}

impl<T: for<'de> Deserialize<'de>> Decoder for PostcardPrefixCodec<T> {
    type Item = T;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }

        // Try to decode the length prefix
        let length_result = postcard::from_bytes::<usize>(&src);
        let (length, length_size) = match length_result {
            Ok(length) => {
                // Calculate how many bytes were consumed for the length
                let length_bytes = postcard::to_stdvec(&length)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                (length, length_bytes.len())
            }
            Err(_) => {
                // Not enough bytes to decode length yet
                return Ok(None);
            }
        };

        // Check if we have enough bytes for the complete message
        if src.len() < length_size + length {
            return Ok(None);
        }

        // Extract the frame (skip length prefix)
        let frame = src.split_to(length_size + length);
        let message_bytes = &frame[length_size..];

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
