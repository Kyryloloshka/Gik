use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct Hash(pub [u8; 20]);

impl Hash {
    pub fn from_hex(s: &str) -> std::result::Result<Self, String> {
        let bytes = hex::decode(s).map_err(|e| e.to_string())?;
        let array: [u8; 20] = bytes.try_into().map_err(|_| "Invalid hash length".to_string())?;
        Ok(Hash(array))
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl From<[u8; 20]> for Hash {
    fn from(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }
}

impl AsRef<[u8; 20]> for Hash {
    fn as_ref(&self) -> &[u8; 20] {
        &self.0
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_display() {
        let h = Hash([0xde, 0xad, 0xbe, 0xef, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
        assert_eq!(format!("{}", h), "deadbeef00112233445566778899aabbccddeeff");
    }

    #[test]
    fn test_hash_from_bytes() {
        let bytes = [1u8; 20];
        let h = Hash::from(bytes);
        assert_eq!(h.0, bytes);
    }
}
