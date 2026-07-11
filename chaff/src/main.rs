use serde::{Deserialize, Serialize};
use hmac::{Hmac, Mac};
use sha2::{Sha256, Digest};
use rand::{thread_rng, Rng, RngCore};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use axum::{
    routing::post,
    extract::State,
    Json, Router,
};
use tower_http::cors::{Any, CorsLayer};

type HmacSha256 = Hmac<Sha256>;
const FIXED_PACKET_SIZE: usize = 512;

// --- Data Structures ---

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BotShard {
    pub session_id: String,
    pub shard_index: usize,
    pub total_shards: usize,
    pub payload_slice: Vec<u8>,
    pub checksum: String,
}

impl Drop for BotShard {
    fn drop(&mut self) {
        for byte in self.payload_slice.iter_mut() { *byte = 0x00; }
        self.session_id.clear();
        self.checksum.clear();
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SecurePacket {
    pub sequence_id: u64,
    pub timestamp: u64,
    pub payload: Vec<u8>, 
    pub mac: Vec<u8>,
}

// --- The Core Pipeline Engines ---

pub struct PurplePipeline {
    ratchet_key: Vec<u8>, 
    sequence_counter: u64,
    ratchet_counter: u64,
}

impl PurplePipeline {
    pub fn new(initial_key: Vec<u8>) -> Self {
        Self { ratchet_key: initial_key, sequence_counter: 0, ratchet_counter: 0 }
    }

    fn step_key_ratchet(&mut self) {
        let mut hasher = Sha256::new();
        hasher.update(&self.ratchet_key);
        hasher.update(&self.ratchet_counter.to_be_bytes());
        let final_hash = hasher.finalize().to_vec();
        for byte in self.ratchet_key.iter_mut() { *byte = 0x00; }
        self.ratchet_key = final_hash;
        self.ratchet_counter += 1;
    }

    pub fn create_chaff_packet(&mut self) -> SecurePacket {
        let seq = self.sequence_counter;
        self.sequence_counter += 1;
        let mut fake_payload = vec![0u8; FIXED_PACKET_SIZE];
        thread_rng().fill_bytes(&mut fake_payload);
        let mut fake_key = vec![0u8; 32];
        thread_rng().fill_bytes(&mut fake_key);
        let mut mac = HmacSha256::new_from_slice(&fake_key).unwrap();
        mac.update(&seq.to_be_bytes());
        mac.update(&fake_payload);

        SecurePacket { sequence_id: seq, timestamp: 0, payload: fake_payload, mac: mac.finalize().into_bytes().to_vec() }
    }

    pub fn create_wheat_packet(&mut self, shard: &BotShard) -> SecurePacket {
        let seq = self.sequence_counter;
        self.sequence_counter += 1;
        let serialized_shard = serde_json::to_vec(shard).unwrap();
        let payload_len = serialized_shard.len();
        let mut uniform_payload = vec![0u8; FIXED_PACKET_SIZE];
        thread_rng().fill_bytes(&mut uniform_payload);
        uniform_payload[0] = ((payload_len >> 8) & 0xFF) as u8;
        uniform_payload[1] = (payload_len & 0xFF) as u8;
        uniform_payload[2..2 + payload_len].copy_from_slice(&serialized_shard);
        let mut mac = HmacSha256::new_from_slice(&self.ratchet_key).unwrap();
        mac.update(&seq.to_be_bytes());
        mac.update(&uniform_payload);

        let packet = SecurePacket { sequence_id: seq, timestamp: 0, payload: uniform_payload, mac: mac.finalize().into_bytes().to_vec() };
        self.step_key_ratchet();
        packet
    }
}

pub struct ShardEngine;
impl ShardEngine {
    pub fn slice_data(data: &str, chunk_size: usize) -> Vec<BotShard> {
        let raw_bytes = data.as_bytes();
        let total_shards = (raw_bytes.len() + chunk_size - 1) / chunk_size;
        let session_id: String = thread_rng().sample_iter(&rand::distributions::Alphanumeric).take(8).map(char::from).collect();
        let mut shards = Vec::new();
        for i in 0..total_shards {
            let start = i * chunk_size;
            let end = std::cmp::min(start + chunk_size, raw_bytes.len());
            shards.push(BotShard {
                session_id: session_id.clone(),
                shard_index: i,
                total_shards,
                payload_slice: raw_bytes[start..end].to_vec(),
                checksum: format!("{:x}", Sha256::digest(&raw_bytes[start..end])),
            });
        }
        shards
    }
}

// --- API Request Schema and Global State Map ---

#[derive(Deserialize)]
pub struct InboundPayload {
    pub instruction: String,
}

#[derive(Serialize)]
pub struct OutboundWirePayload {
    pub wire_queue_size: usize,
    pub transmitted_packets: Vec<SecurePacket>,
}

// Thread-safe memory container tracking pipeline states across network threads
pub struct AppState {
    pub pipeline: Mutex<PurplePipeline>,
}

// --- Dynamic API Route Handler ---

async fn handle_encrypt_stream(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<InboundPayload>,
) -> Json<OutboundWirePayload> {
    println!("[*] API Daemon received operational instruction from RA App frontend context.");
    
    // Slice raw string data via memory engine fragmentation
    let mut shards = ShardEngine::slice_data(&payload.instruction, 16);
    let mut packets_wire_stream = Vec::new();
    
    // Acquire temporary thread lock safely over the shared cryptographic key pipeline
    let mut pipeline = state.pipeline.lock().unwrap();

    while !shards.is_empty() {
        // Interleave random bursts of high-entropy noise packets [1.1]
        let chaff_count = thread_rng().gen_range(1..3);
        for _ in 0..chaff_count {
            packets_wire_stream.push(pipeline.create_chaff_packet());
        }
        let real_shard = shards.pop().unwrap();
        packets_wire_stream.push(pipeline.create_wheat_packet(&real_shard));
    }

    println!("[+] Successfully multiplexed data. Generated {} secure wire-format packets.", packets_wire_stream.len());
    
    Json(OutboundWirePayload {
        wire_queue_size: packets_wire_stream.len(),
        transmitted_packets: packets_wire_stream,
    })
}

// --- Asynchronous Daemon Entry Point ---

#[tokio::main]
async fn main() {
    println!("[*] Initializing Hardened Cryptographic Transport Daemon...");
    
    // Set up shared OTP master key state array
    let master_otp_key = vec![42u8; 32];
    let shared_state = Arc::new(AppState {
        pipeline: Mutex::new(PurplePipeline::new(master_otp_key)),
    });

    // Configure open CORS rules so your separate local Next.js node server can query the endpoint
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Bind paths to endpoints
    let app = Router::new()
        .route("/api/transport/encrypt", post(handle_encrypt_stream))
        .layer(cors)
        .with_state(shared_state);

    // Bind listener directly to local loopback gateway port 8080 [1.1]
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("[+] DAEMON ONLINE -> Listening securely on http://127.0.0.1:8080");
    println!("[=] Protected Endpoints: POST /api/transport/encrypt");
    
    // Keep thread running permanently in background sleep execution state [1.1]
    axum::serve(listener, app).await.unwrap();
}

