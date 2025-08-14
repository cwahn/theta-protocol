use bytes::BytesMut;
use serde::{Deserialize, Serialize};
use std::{
    format,
    println,
    string::{String, ToString},
    time::{Duration, Instant},
    vec,
    vec::Vec,
};
use tokio_util::codec::{Decoder, Encoder};

use crate::codec::{postcard_cobs::PostcardCobsCodec, postcard_prefix::PostcardPrefixCodec};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct SmallMessage {
    id: u32,
    value: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct MediumMessage {
    id: u32,
    data: Vec<u8>,
    text: String,
    flags: Vec<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct LargeMessage {
    id: u32,
    payload: Vec<u8>,
    metadata: std::collections::HashMap<String, String>,
    nested: Vec<MediumMessage>,
}

fn generate_small_messages(count: usize) -> Vec<SmallMessage> {
    (0..count)
        .map(|i| SmallMessage {
            id: i as u32,
            value: (i * 13) as u64,
        })
        .collect()
}

fn generate_medium_messages(count: usize) -> Vec<MediumMessage> {
    (0..count)
        .map(|i| MediumMessage {
            id: i as u32,
            data: (0..100).map(|j| ((i + j) % 256) as u8).collect(),
            text: format!("Message number {}", i),
            flags: (0..20).map(|j| (i + j) % 2 == 0).collect(),
        })
        .collect()
}

fn generate_large_messages(count: usize) -> Vec<LargeMessage> {
    (0..count)
        .map(|i| {
            let mut metadata = std::collections::HashMap::new();
            metadata.insert(format!("key_{}", i), format!("value_{}", i));
            metadata.insert("timestamp".to_string(), format!("{}", i * 1000));
            
            LargeMessage {
                id: i as u32,
                payload: (0..1000).map(|j| ((i + j) % 256) as u8).collect(),
                metadata,
                nested: generate_medium_messages(5),
            }
        })
        .collect()
}

fn benchmark_codec<T, C>(codec: &mut C, messages: &[T], name: &str) -> (Duration, Duration, usize)
where
    T: Clone + serde::Serialize + for<'de> serde::Deserialize<'de> + PartialEq + std::fmt::Debug,
    C: Encoder<T> + Decoder<Item = T>,
    <C as Encoder<T>>::Error: std::fmt::Debug,
    <C as Decoder>::Error: std::fmt::Debug,
{
    // Encode benchmark
    let mut encode_buffer = BytesMut::new();
    let encode_start = Instant::now();
    
    for message in messages {
        codec.encode(message.clone(), &mut encode_buffer).unwrap();
    }
    
    let encode_duration = encode_start.elapsed();
    let encoded_size = encode_buffer.len();

    // Decode benchmark
    let decode_start = Instant::now();
    let mut decoded_messages = Vec::new();
    
    while let Some(msg) = codec.decode(&mut encode_buffer).unwrap() {
        decoded_messages.push(msg);
    }
    
    let decode_duration = decode_start.elapsed();

    // Verify correctness
    assert_eq!(messages.len(), decoded_messages.len());
    for (original, decoded) in messages.iter().zip(decoded_messages.iter()) {
        assert_eq!(original, decoded);
    }

    println!(
        "{}: Encode: {:?}, Decode: {:?}, Size: {} bytes",
        name, encode_duration, decode_duration, encoded_size
    );

    (encode_duration, decode_duration, encoded_size)
}

#[tokio::test]
async fn performance_comparison_small_messages() {
    let messages = generate_small_messages(10000);
    println!("\n=== Small Messages Performance (10,000 messages) ===");

    let mut cobs_codec = PostcardCobsCodec::<SmallMessage>::default();
    let (cobs_encode, cobs_decode, cobs_size) = 
        benchmark_codec(&mut cobs_codec, &messages, "COBS");

    let mut prefix_codec = PostcardPrefixCodec::<SmallMessage>::default();
    let (prefix_encode, prefix_decode, prefix_size) = 
        benchmark_codec(&mut prefix_codec, &messages, "Prefix");

    println!("Size difference: {} bytes ({:.2}%)", 
        prefix_size as i64 - cobs_size as i64,
        ((prefix_size as f64 - cobs_size as f64) / cobs_size as f64) * 100.0
    );
    println!("Encode speed ratio (Prefix/COBS): {:.2}", 
        prefix_encode.as_nanos() as f64 / cobs_encode.as_nanos() as f64);
    println!("Decode speed ratio (Prefix/COBS): {:.2}", 
        prefix_decode.as_nanos() as f64 / cobs_decode.as_nanos() as f64);
}

#[tokio::test]
async fn performance_comparison_medium_messages() {
    let messages = generate_medium_messages(1000);
    println!("\n=== Medium Messages Performance (1,000 messages) ===");

    let mut cobs_codec = PostcardCobsCodec::<MediumMessage>::default();
    let (cobs_encode, cobs_decode, cobs_size) = 
        benchmark_codec(&mut cobs_codec, &messages, "COBS");

    let mut prefix_codec = PostcardPrefixCodec::<MediumMessage>::default();
    let (prefix_encode, prefix_decode, prefix_size) = 
        benchmark_codec(&mut prefix_codec, &messages, "Prefix");

    println!("Size difference: {} bytes ({:.2}%)", 
        prefix_size as i64 - cobs_size as i64,
        ((prefix_size as f64 - cobs_size as f64) / cobs_size as f64) * 100.0
    );
    println!("Encode speed ratio (Prefix/COBS): {:.2}", 
        prefix_encode.as_nanos() as f64 / cobs_encode.as_nanos() as f64);
    println!("Decode speed ratio (Prefix/COBS): {:.2}", 
        prefix_decode.as_nanos() as f64 / cobs_decode.as_nanos() as f64);
}

#[tokio::test]
async fn performance_comparison_large_messages() {
    let messages = generate_large_messages(100);
    println!("\n=== Large Messages Performance (100 messages) ===");

    let mut cobs_codec = PostcardCobsCodec::<LargeMessage>::default();
    let (cobs_encode, cobs_decode, cobs_size) = 
        benchmark_codec(&mut cobs_codec, &messages, "COBS");

    let mut prefix_codec = PostcardPrefixCodec::<LargeMessage>::default();
    let (prefix_encode, prefix_decode, prefix_size) = 
        benchmark_codec(&mut prefix_codec, &messages, "Prefix");

    println!("Size difference: {} bytes ({:.2}%)", 
        prefix_size as i64 - cobs_size as i64,
        ((prefix_size as f64 - cobs_size as f64) / cobs_size as f64) * 100.0
    );
    println!("Encode speed ratio (Prefix/COBS): {:.2}", 
        prefix_encode.as_nanos() as f64 / cobs_encode.as_nanos() as f64);
    println!("Decode speed ratio (Prefix/COBS): {:.2}", 
        prefix_decode.as_nanos() as f64 / cobs_decode.as_nanos() as f64);
}

#[tokio::test]
async fn overhead_analysis() {
    println!("\n=== Overhead Analysis ===");

    // Test with varying message sizes to understand overhead patterns
    let message_sizes = vec![10, 50, 100, 500, 1000, 5000];
    
    for size in message_sizes {
        let message = MediumMessage {
            id: 1,
            data: vec![0x42; size],
            text: "test".to_string(),
            flags: vec![true; 10],
        };

        let mut cobs_buffer = BytesMut::new();
        let mut cobs_codec = PostcardCobsCodec::<MediumMessage>::default();
        cobs_codec.encode(message.clone(), &mut cobs_buffer).unwrap();

        let mut prefix_buffer = BytesMut::new();
        let mut prefix_codec = PostcardPrefixCodec::<MediumMessage>::default();
        prefix_codec.encode(message.clone(), &mut prefix_buffer).unwrap();

        let raw_size = postcard::to_stdvec(&message).unwrap().len();
        let cobs_overhead = cobs_buffer.len() - raw_size;
        let prefix_overhead = prefix_buffer.len() - raw_size;

        println!(
            "Payload: {} bytes, Raw: {} bytes, COBS: {} bytes (+{}), Prefix: {} bytes (+{})",
            size, raw_size, cobs_buffer.len(), cobs_overhead, prefix_buffer.len(), prefix_overhead
        );
    }
}
