use bytes::BytesMut;
use chrono::{DateTime, Utc};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use fake::{
    faker::{
        internet::en::{IPv4, Username},
        lorem::en::Sentence,
        name::en::{FirstName, LastName},
    },
    Fake,
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    string::String,
    vec::Vec,
};
use theta_protocol::codec::{
    postcard_cobs::PostcardCobsCodec, postcard_prefix::PostcardPrefixCodec,
};
use tokio_util::codec::{Decoder, Encoder};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    level: String,
    message: String,
    module: String,
    thread_id: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct UserProfile {
    id: Uuid,
    username: String,
    first_name: String,
    last_name: String,
    email: String,
    ip_address: String,
    metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct NetworkPacket {
    id: u64,
    source_ip: String,
    dest_ip: String,
    protocol: u8,
    payload: Vec<u8>,
    headers: HashMap<String, Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct IoTSensorData {
    device_id: Uuid,
    timestamp: DateTime<Utc>,
    temperature: i16,  // Celsius * 100
    humidity: u16,     // Percentage * 100
    pressure: u32,     // Pascal
    battery_level: u8, // Percentage
    raw_data: Vec<u8>, // Variable sensor readings
}

// Data generation functions with controlled randomness
fn generate_log_entries(count: usize, seed: u64) -> Vec<LogEntry> {
    let mut rng = StdRng::seed_from_u64(seed);
    let levels = ["DEBUG", "INFO", "WARN", "ERROR", "TRACE"];
    let modules = ["auth", "database", "network", "cache", "worker"];
    
    (0..count)
        .map(|i| LogEntry {
            timestamp: DateTime::from_timestamp(1700000000 + i as i64 * 60, 0).unwrap(),
            level: levels[rng.gen_range(0..levels.len())].to_string(),
            message: Sentence(1..3).fake_with_rng(&mut rng),
            module: modules[rng.gen_range(0..modules.len())].to_string(),
            thread_id: rng.gen_range(1..=100),
        })
        .collect()
}

fn generate_user_profiles(count: usize, seed: u64) -> Vec<UserProfile> {
    let mut rng = StdRng::seed_from_u64(seed);
    
    (0..count)
        .map(|_| {
            let mut metadata = HashMap::new();
            metadata.insert("last_login".to_string(), "2024-01-01T00:00:00Z".to_string());
            metadata.insert("preferences".to_string(), "{\"theme\":\"dark\",\"notifications\":true}".to_string());
            metadata.insert("role".to_string(), "user".to_string());
            
            UserProfile {
                id: Uuid::new_v4(),
                username: Username().fake_with_rng(&mut rng),
                first_name: FirstName().fake_with_rng(&mut rng),
                last_name: LastName().fake_with_rng(&mut rng),
                email: format!("user{}@example.com", rng.gen_range(1000..9999)),
                ip_address: IPv4().fake_with_rng(&mut rng),
                metadata,
            }
        })
        .collect()
}

fn generate_network_packets(count: usize, seed: u64) -> Vec<NetworkPacket> {
    let mut rng = StdRng::seed_from_u64(seed);
    
    (0..count)
        .map(|i| {
            let payload_size = rng.gen_range(64..1500); // Realistic packet sizes
            let mut payload = vec![0u8; payload_size];
            rng.fill(&mut payload[..]);
            
            let mut headers = HashMap::new();
            headers.insert("content-type".to_string(), b"application/octet-stream".to_vec());
            headers.insert("user-agent".to_string(), b"BenchClient/1.0".to_vec());
            
            NetworkPacket {
                id: i as u64,
                source_ip: IPv4().fake_with_rng(&mut rng),
                dest_ip: IPv4().fake_with_rng(&mut rng),
                protocol: rng.gen_range(1..=255),
                payload,
                headers,
            }
        })
        .collect()
}

fn generate_iot_sensor_data(count: usize, seed: u64) -> Vec<IoTSensorData> {
    let mut rng = StdRng::seed_from_u64(seed);
    
    (0..count)
        .map(|i| {
            let raw_data_size = rng.gen_range(16..256); // Sensor data varies
            let mut raw_data = vec![0u8; raw_data_size];
            rng.fill(&mut raw_data[..]);
            
            IoTSensorData {
                device_id: Uuid::new_v4(),
                timestamp: DateTime::from_timestamp(1700000000 + i as i64 * 30, 0).unwrap(),
                temperature: rng.gen_range(-4000..8500), // -40°C to 85°C
                humidity: rng.gen_range(0..10000),        // 0% to 100%
                pressure: rng.gen_range(80000..120000),   // Atmospheric pressure range
                battery_level: rng.gen_range(0..=100),
                raw_data,
            }
        })
        .collect()
}

// Focused benchmark functions
fn bench_encode_decode<T, C>(
    c: &mut Criterion,
    data: &[T],
    codec_name: &str,
    data_type: &str,
) where
    T: Clone + Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + PartialEq,
    C: Default + Encoder<T> + Decoder<Item = T>,
    <C as Encoder<T>>::Error: std::fmt::Debug,
    <C as Decoder>::Error: std::fmt::Debug,
{
    let mut group = c.benchmark_group(format!("{}_{}", data_type, codec_name));
    
    // Calculate throughput based on raw data size
    let total_raw_size: usize = data.iter()
        .map(|item| postcard::to_stdvec(item).unwrap().len())
        .sum();
    group.throughput(Throughput::Bytes(total_raw_size as u64));
    
    // Encoding benchmark
    group.bench_function("encode", |b| {
        b.iter(|| {
            let mut codec = C::default();
            let mut buffer = BytesMut::new();
            for item in data {
                codec.encode(black_box(item.clone()), &mut buffer).unwrap();
            }
            black_box(buffer)
        });
    });
    
    // Decoding benchmark
    group.bench_function("decode", |b| {
        let mut codec = C::default();
        let mut encoded_buffer = BytesMut::new();
        for item in data {
            codec.encode(item.clone(), &mut encoded_buffer).unwrap();
        }
        
        b.iter(|| {
            let mut codec = C::default();
            let mut buffer = encoded_buffer.clone();
            let mut decoded = Vec::new();
            while let Some(item) = codec.decode(&mut buffer).unwrap() {
                decoded.push(black_box(item));
            }
            decoded
        });
    });
    
    group.finish();
}

fn bench_log_entries(c: &mut Criterion) {
    let data = generate_log_entries(50, 42);
    
    bench_encode_decode::<LogEntry, PostcardCobsCodec<LogEntry>>(
        c, &data, "cobs", "log_entries"
    );
    bench_encode_decode::<LogEntry, PostcardPrefixCodec<LogEntry>>(
        c, &data, "prefix", "log_entries"
    );
}

fn bench_user_profiles(c: &mut Criterion) {
    let data = generate_user_profiles(25, 42);
    
    bench_encode_decode::<UserProfile, PostcardCobsCodec<UserProfile>>(
        c, &data, "cobs", "user_profiles"
    );
    bench_encode_decode::<UserProfile, PostcardPrefixCodec<UserProfile>>(
        c, &data, "prefix", "user_profiles"
    );
}

fn bench_network_packets(c: &mut Criterion) {
    let data = generate_network_packets(10, 42);
    
    bench_encode_decode::<NetworkPacket, PostcardCobsCodec<NetworkPacket>>(
        c, &data, "cobs", "network_packets"
    );
    bench_encode_decode::<NetworkPacket, PostcardPrefixCodec<NetworkPacket>>(
        c, &data, "prefix", "network_packets"
    );
}

fn bench_iot_sensor_data(c: &mut Criterion) {
    let data = generate_iot_sensor_data(50, 42);
    
    bench_encode_decode::<IoTSensorData, PostcardCobsCodec<IoTSensorData>>(
        c, &data, "cobs", "iot_sensor_data"
    );
    bench_encode_decode::<IoTSensorData, PostcardPrefixCodec<IoTSensorData>>(
        c, &data, "prefix", "iot_sensor_data"
    );
}

// Special test for zero-byte sensitivity (COBS characteristic)
fn bench_zero_byte_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("zero_byte_patterns");
    
    // Test different zero byte densities
    let zero_densities = [0.0, 0.1, 0.25, 0.5]; // 0%, 10%, 25%, 50% zeros
    
    for &density in &zero_densities {
        let mut rng = StdRng::seed_from_u64(42);
        let data_size = 1000;
        let mut payload = vec![0u8; data_size];
        
        // Fill with random data, then replace some with zeros
        rng.fill(&mut payload[..]);
        let zero_count = (data_size as f32 * density) as usize;
        for _i in 0..zero_count {
            let pos = rng.gen_range(0..data_size);
            payload[pos] = 0;
        }
        
        let packet = NetworkPacket {
            id: 1,
            source_ip: "192.168.1.1".to_string(),
            dest_ip: "192.168.1.2".to_string(),
            protocol: 6,
            payload,
            headers: HashMap::new(),
        };
        
        let data = vec![packet];
        let raw_size = postcard::to_stdvec(&data[0]).unwrap().len();
        group.throughput(Throughput::Bytes(raw_size as u64));
        
        // Test COBS
        group.bench_with_input(
            BenchmarkId::new("cobs", format!("{:.0}%_zeros", density * 100.0)),
            &data,
            |b, data| {
                b.iter(|| {
                    let mut codec = PostcardCobsCodec::<NetworkPacket>::default();
                    let mut buffer = BytesMut::new();
                    codec.encode(black_box(data[0].clone()), &mut buffer).unwrap();
                    black_box(buffer)
                });
            },
        );
        
        // Test Prefix
        group.bench_with_input(
            BenchmarkId::new("prefix", format!("{:.0}%_zeros", density * 100.0)),
            &data,
            |b, data| {
                b.iter(|| {
                    let mut codec = PostcardPrefixCodec::<NetworkPacket>::default();
                    let mut buffer = BytesMut::new();
                    codec.encode(black_box(data[0].clone()), &mut buffer).unwrap();
                    black_box(buffer)
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_log_entries,
    bench_user_profiles,
    bench_network_packets,
    bench_iot_sensor_data,
    bench_zero_byte_patterns
);
criterion_main!(benches);
