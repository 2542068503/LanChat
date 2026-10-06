use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

pub fn compute_sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub fn compute_sha256_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    // Use a 2MB buffer for faster file I/O
    let mut buffer = vec![0u8; 2 * 1024 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_sha256_bytes() {
        let empty_hash = compute_sha256_bytes(b"");
        assert_eq!(
            empty_hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );

        let hello_hash = compute_sha256_bytes(b"hello");
        assert_eq!(
            hello_hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_compute_sha256_file() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("lanchat_sha_test.tmp");
        let content = b"LanChat file hashing verification test 123456";
        std::fs::write(&test_file, content).expect("write temp file");

        let file_hash = compute_sha256_file(&test_file).expect("compute file hash");
        let byte_hash = compute_sha256_bytes(content);
        assert_eq!(file_hash, byte_hash);

        let _ = std::fs::remove_file(test_file);
    }
}
