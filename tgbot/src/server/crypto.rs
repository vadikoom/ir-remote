use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305, XNonce,
};

use super::{Error, Result};

pub const COMMAND: u8 = 1;
pub const STATUS: u8 = 2;
pub const MAX_PACKET: usize = 2048;
const HEADER: usize = 2;
const NONCE: usize = 24;
const TAG: usize = 16;
const OVERHEAD: usize = HEADER + NONCE + TAG;

#[derive(Clone)]
pub struct Crypto {
    cipher: XChaCha20Poly1305,
}

impl Crypto {
    pub fn from_hex(secret: &str) -> Result<Self> {
        if secret.len() != 64 || !secret.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err(Error::Security(
                "IR_SHARED_SECRET must contain exactly 64 hexadecimal characters",
            ));
        }
        let mut key = [0; 32];
        for (i, byte) in key.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&secret[i * 2..i * 2 + 2], 16)
                .map_err(|_| Error::Security("Invalid shared key"))?;
        }
        Ok(Self {
            cipher: XChaCha20Poly1305::new((&key).into()),
        })
    }

    pub fn seal(&self, kind: u8, plaintext: &[u8]) -> Result<Vec<u8>> {
        let mut nonce = [0; NONCE];
        getrandom::fill(&mut nonce)
            .map_err(|_| Error::Security("Secure random source unavailable"))?;
        self.seal_with_nonce(kind, plaintext, nonce)
    }

    fn seal_with_nonce(&self, kind: u8, plaintext: &[u8], nonce: [u8; NONCE]) -> Result<Vec<u8>> {
        if plaintext.len() > MAX_PACKET - OVERHEAD {
            return Err(Error::Security("Packet exceeds firmware capacity"));
        }
        let header = [1, kind];
        let ciphertext = self
            .cipher
            .encrypt(
                XNonce::from_slice(&nonce),
                Payload {
                    msg: plaintext,
                    aad: &header,
                },
            )
            .map_err(|_| Error::Security("Encryption failed"))?;
        let mut packet = Vec::with_capacity(OVERHEAD + plaintext.len());
        packet.extend_from_slice(&header);
        packet.extend_from_slice(&nonce);
        packet.extend_from_slice(&ciphertext);
        Ok(packet)
    }

    pub fn open(&self, kind: u8, packet: &[u8]) -> Result<Vec<u8>> {
        if !(OVERHEAD..=MAX_PACKET).contains(&packet.len()) || packet[..HEADER] != [1, kind] {
            return Err(Error::Security("Invalid encrypted packet header or length"));
        }
        self.cipher
            .decrypt(
                XNonce::from_slice(&packet[HEADER..HEADER + NONCE]),
                Payload {
                    msg: &packet[HEADER + NONCE..],
                    aad: &packet[..HEADER],
                },
            )
            .map_err(|_| Error::Security("Packet authentication failed"))
    }
}

#[cfg(test)]
pub const TEST_SECRET: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(hex: &str) -> Vec<u8> {
        let hex = hex.trim();
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect()
    }

    #[test]
    fn matches_monocypher_firmware_packets_in_both_directions() {
        let crypto = Crypto::from_hex(TEST_SECRET).unwrap();
        let nonce = std::array::from_fn(|i| i as u8);
        let command = br#"{"sequence":1,"data":[100,200]}"#;
        let status = br#"{"last_command_sequence_number":42}"#;
        let command_packet = fixture(include_str!("../../testdata/command.hex"));
        let status_packet = fixture(include_str!("../../testdata/status.hex"));
        assert_eq!(
            crypto.seal_with_nonce(COMMAND, command, nonce).unwrap(),
            command_packet
        );
        assert_eq!(
            crypto.seal_with_nonce(STATUS, status, nonce).unwrap(),
            status_packet
        );
        assert_eq!(crypto.open(COMMAND, &command_packet).unwrap(), command);
        assert_eq!(crypto.open(STATUS, &status_packet).unwrap(), status);
    }

    #[test]
    fn rejects_tampering_wrong_keys_directions_and_plaintext() {
        let crypto = Crypto::from_hex(TEST_SECRET).unwrap();
        let packet = crypto
            .seal(COMMAND, br#"{"sequence":1,"data":[100,200]}"#)
            .unwrap();
        assert_eq!(
            crypto.open(COMMAND, &packet).unwrap(),
            br#"{"sequence":1,"data":[100,200]}"#
        );
        for i in 0..packet.len() {
            let mut corrupt = packet.clone();
            corrupt[i] ^= 1;
            assert!(
                crypto.open(COMMAND, &corrupt).is_err(),
                "accepted modified byte {i}"
            );
        }
        assert!(crypto.open(STATUS, &packet).is_err());
        assert!(Crypto::from_hex(&"ff".repeat(32))
            .unwrap()
            .open(COMMAND, &packet)
            .is_err());
        assert!(crypto.open(COMMAND, b"{}").is_err());
        for len in 0..packet.len() {
            assert!(crypto.open(COMMAND, &packet[..len]).is_err());
        }
    }

    #[test]
    fn validates_keys_bounds_and_random_nonces() {
        for key in ["", "password", &"g".repeat(64), &"é".repeat(32)] {
            assert!(Crypto::from_hex(key).is_err());
        }
        let crypto = Crypto::from_hex(TEST_SECRET).unwrap();
        assert_ne!(
            crypto.seal(STATUS, b"{}").unwrap(),
            crypto.seal(STATUS, b"{}").unwrap()
        );
        assert!(crypto
            .seal(COMMAND, &vec![0; MAX_PACKET - OVERHEAD + 1])
            .is_err());
        let packet = crypto
            .seal(COMMAND, &vec![0; MAX_PACKET - OVERHEAD])
            .unwrap();
        assert_eq!(packet.len(), MAX_PACKET);
        assert_eq!(
            crypto.open(COMMAND, &packet).unwrap().len(),
            MAX_PACKET - OVERHEAD
        );
    }
}
