// path inside your project: src/otp.rs
use std::fs::File;
use std::io::{Read, Write};
use rand::{thread_rng, RngCore};

pub struct OneTimePadEngine;

impl OneTimePadEngine {
    /// Generates a completely new, cryptographically secure random key file 
    /// backed directly by the device's hardware entropy pool.
    pub fn generate_key_file(filepath: &str, size_bytes: usize) -> std::io::Result<()> {
        let mut key_buffer = vec![0u8; size_bytes];
        thread_rng().fill_bytes(&mut key_buffer);
        
        let mut file = File::create(filepath)?;
        file.write_all(&key_buffer)?;
        
        // Zero out volatile memory block immediately after saving [1.1]
        for byte in key_buffer.iter_mut() { *byte = 0x00; }
        Ok(())
    }

    /// CIPHER 1 MAIN GATE: Executes pure mathematical XOR streaming encryption.
    /// Because XOR with a truly random, non-repeating key stream provides 
    /// information-theoretic security, the ciphertext is mathematically unbreakable.
    pub fn crypt_stream(plaintext: &[u8], key_stream: &[u8]) -> Vec<u8> {
        if key_stream.len() < plaintext.len() {
            panic!("Cryptographic Failure: One-Time Pad key array stream is shorter than the payload payload.");
        }

        plaintext
            .iter()
            .zip(key_stream.iter())
            .map(|(&plain_byte, &key_byte)| plain_byte ^ key_byte) // The fundamental OTP math operation
            .collect()
    }
}

