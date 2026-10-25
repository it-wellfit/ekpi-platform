#![forbid(unsafe_code)]

//! WORM evidence storage (Master Prompt Phase 9a).
//!
//! Append-only storage with SHA-256 hash chain. Tamper detection.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A WORM entry with hash chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WormEntry {
    pub id: String,
    pub data: String,
    pub prev_hash: String,
    pub hash: String,
    pub timestamp: String,
}

impl WormEntry {
    pub fn new(id: &str, data: &str, prev_hash: &str) -> Self {
        let hash = Self::compute_hash(id, data, prev_hash);
        WormEntry {
            id: id.to_string(),
            data: data.to_string(),
            prev_hash: prev_hash.to_string(),
            hash,
            timestamp: String::new(),
        }
    }

    fn compute_hash(id: &str, data: &str, prev_hash: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(id.as_bytes());
        hasher.update(data.as_bytes());
        hasher.update(prev_hash.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// WORM store with hash chain integrity.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WormStore {
    pub entries: Vec<WormEntry>,
}

impl WormStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, id: &str, data: &str) -> WormEntry {
        let prev_hash = self
            .entries
            .last()
            .map(|e| e.hash.clone())
            .unwrap_or_else(|| "GENESIS".to_string());
        let entry = WormEntry::new(id, data, &prev_hash);
        self.entries.push(entry.clone());
        entry
    }

    pub fn verify_chain(&self) -> bool {
        if self.entries.is_empty() {
            return true;
        }
        // First entry must have GENESIS as prev_hash
        if self.entries[0].prev_hash != "GENESIS" {
            return false;
        }
        for i in 1..self.entries.len() {
            if self.entries[i].prev_hash != self.entries[i - 1].hash {
                return false;
            }
            // Verify hash computation
            let expected = WormEntry::compute_hash(
                &self.entries[i].id,
                &self.entries[i].data,
                &self.entries[i].prev_hash,
            );
            if expected != self.entries[i].hash {
                return false;
            }
        }
        // Verify first entry hash
        let expected = WormEntry::compute_hash(
            &self.entries[0].id,
            &self.entries[0].data,
            &self.entries[0].prev_hash,
        );
        expected == self.entries[0].hash
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn detect_tampering(&self) -> Vec<usize> {
        let mut tampered = Vec::new();
        for i in 0..self.entries.len() {
            let prev_hash = if i == 0 {
                "GENESIS".to_string()
            } else {
                self.entries[i - 1].hash.clone()
            };
            let expected =
                WormEntry::compute_hash(&self.entries[i].id, &self.entries[i].data, &prev_hash);
            if expected != self.entries[i].hash || self.entries[i].prev_hash != prev_hash {
                tampered.push(i);
            }
        }
        tampered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_and_verify() {
        let mut store = WormStore::new();
        store.append("E1", "data1");
        store.append("E2", "data2");
        store.append("E3", "data3");
        assert!(store.verify_chain());
        assert_eq!(store.len(), 3);
    }

    #[test]
    fn empty_store_verifies() {
        let store = WormStore::new();
        assert!(store.verify_chain());
        assert!(store.is_empty());
    }

    #[test]
    fn tampering_detected() {
        let mut store = WormStore::new();
        store.append("E1", "data1");
        store.append("E2", "data2");
        // Tamper with data
        store.entries[1].data = "tampered".to_string();
        assert!(!store.verify_chain());
        let tampered = store.detect_tampering();
        assert!(tampered.contains(&1));
    }

    #[test]
    fn prev_hash_chain_integrity() {
        let mut store = WormStore::new();
        let e1 = store.append("E1", "data1");
        let e2 = store.append("E2", "data2");
        assert_eq!(e2.prev_hash, e1.hash);
    }

    #[test]
    fn genesis_prev_hash() {
        let mut store = WormStore::new();
        let e1 = store.append("E1", "data1");
        assert_eq!(e1.prev_hash, "GENESIS");
    }
}
