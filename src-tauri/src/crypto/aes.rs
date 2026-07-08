use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;

const PSK: &[u8; 32] = b"LanChat_Secure_Key_2026_00000000";

pub fn encrypt(data: &[u8]) -> Result<Vec<u8>, String> {
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(PSK);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    match cipher.encrypt(nonce, data) {
        Ok(mut ciphertext) => {
            let mut result = Vec::with_capacity(12 + ciphertext.len());
            result.extend_from_slice(&nonce_bytes);
            result.append(&mut ciphertext);
            Ok(result)
        }
        Err(e) => Err(format!("Encryption failed: {}", e)),
    }
}

pub fn decrypt(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < 12 {
        return Err("Data too short".into());
    }
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(PSK);
    let cipher = Aes256Gcm::new(key);

    let nonce = Nonce::from_slice(&data[0..12]);
    let ciphertext = &data[12..];

    match cipher.decrypt(nonce, ciphertext) {
        Ok(plaintext) => Ok(plaintext),
        Err(e) => Err(format!("Decryption failed: {}", e)),
    }
}
