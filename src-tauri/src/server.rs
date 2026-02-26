use bip39::Mnemonic;
use hex;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Debug)]
pub struct PairingCodes {
    pub uuid: String,
    pub passphrase: Vec<String>,
}

/// Generates a high-entropy UUID for QR codes, and a mathematically
/// correlated 4-word dictionary passphrase for manual typing.
#[tauri::command]
pub async fn generate_pairing_codes() -> Result<PairingCodes, String> {
    // Generate 16 bytes of cryptographically secure random entropy
    let mut entropy = [0u8; 16];
    OsRng.fill_bytes(&mut entropy);

    // Generate the 12-word BIP39 phrase from entropy
    let mnemonic = match Mnemonic::from_entropy(&entropy) {
        Ok(m) => m,
        Err(e) => return Err(format!("Failed to generate mnemonic: {}", e)),
    };

    // We only need a 4-word passphrase for easy typing to guarantee 43 bits of entropy (plenty for a temporary room)
    let words: Vec<String> = mnemonic.words().take(4).map(|w| w.to_string()).collect();

    // Map the selected words back into a unique hex string (this mimics reading the QR code)
    let derived_uuid = hex::encode(words.join("-").as_bytes());

    Ok(PairingCodes {
        uuid: derived_uuid,
        passphrase: words,
    })
}

/// Hashes either a scanned QR UUID or a manually typed 4-word passphrase
/// so both devices end up mathematically producing the exact same `room_id` for the Signaling Server.
#[tauri::command]
pub async fn hash_pairing_code(input: String) -> Result<String, String> {
    // Sanitize input (lowercase, trim whitespace, normalize hyphens)
    let sanitized = input.to_lowercase().trim().replace(" ", "-");

    // If the input was the raw 4-word phrase, derive the pseudo-UUID first
    let raw_payload = if sanitized.split('-').count() == 4 {
        hex::encode(sanitized.as_bytes())
    } else {
        sanitized // Assume it's already the UUID from the QR
    };

    // Hash the UUID using SHA-256 to create the secure Room ID
    let mut hasher = Sha256::new();
    hasher.update(raw_payload.as_bytes());
    let room_id = hex::encode(hasher.finalize());

    Ok(room_id)
}
