// Protocol Config:
//     n, delta, blocksize, f, id

// Network Config:
//     client_port, map[id]ip

// Crypto Config:
//     algorithm_type, pvt_key, map[id]public_key

use super::{is_valid_replica, ParseError};
use crypto::Algorithm;
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use toml::from_str;
use types::Replica;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    // Node network config
    pub net_map: HashMap<Replica, String>,

    // protocol details
    pub delta: u64,
    pub id: Replica,
    pub num_nodes: usize,
    pub num_faults: usize,
    pub block_size: usize,
    pub client_port: u16,
    pub payload: usize,

    // Crypto primitives
    pub crypto_alg: Algorithm,
    pub pk_map: HashMap<Replica, Vec<u8>>,
    pub secret_key_bytes: Vec<u8>,

    // Biaccumulator setup
    pub bi_pp_map: HashMap<Replica, crypto::EVSSPublicParams381>,
    pub bi_p: Option<crypto::EVSSParams381>,

    pub rand_beacon_parameter: Option<crypto::EVSSParams381>,
    pub rand_beacon_queue: HashMap<Replica, std::collections::VecDeque<crypto::EVSSShare381>>,

    pub rand_beacon_shares: Vec<(Vec<std::collections::VecDeque<crypto::EVSSShare381>>, Vec<crypto::EVSSCommit381>)>,
}

impl Node {
    pub fn validate(&self) -> Result<(), ParseError> {
        if self.net_map.len() != self.num_nodes {
            return Err(ParseError::InvalidMapLen(
                self.num_nodes,
                self.net_map.len(),
            ));
        }
        if 2 * self.num_faults >= self.num_nodes {
            return Err(ParseError::IncorrectFaults(self.num_faults, self.num_nodes));
        }
        for repl in &self.net_map {
            if !is_valid_replica(*repl.0, self.num_nodes) {
                return Err(ParseError::InvalidMapEntry(*repl.0));
            }
        }
        match self.crypto_alg {
            Algorithm::ED25519 => {
                for repl in &self.pk_map {
                    if !is_valid_replica(*repl.0, self.num_nodes) {
                        return Err(ParseError::InvalidMapEntry(*repl.0));
                    }
                    if repl.1.len() != crypto::ED25519_PK_SIZE {
                        return Err(ParseError::InvalidPkSize(repl.1.len()));
                    }
                }
                if self.secret_key_bytes.len() != crypto::ED25519_PVT_SIZE {
                    return Err(ParseError::InvalidSkSize(self.secret_key_bytes.len()));
                }
            }
            Algorithm::SECP256K1 => {
                for repl in &self.pk_map {
                    if !is_valid_replica(*repl.0, self.num_nodes) {
                        return Err(ParseError::InvalidMapEntry(*repl.0));
                    }
                    if repl.1.len() != crypto::SECP256K1_PK_SIZE {
                        return Err(ParseError::InvalidPkSize(repl.1.len()));
                    }
                }
                if self.secret_key_bytes.len() != crypto::SECP256K1_PVT_SIZE {
                    return Err(ParseError::InvalidSkSize(self.secret_key_bytes.len()));
                }
            }
            Algorithm::RSA => {
                // Because unimplemented
                return Err(ParseError::Unimplemented("RSA"));
            }
        }
        Ok(())
    }

    pub fn new() -> Node {
        Node {
            block_size: 0,
            client_port: 0,
            crypto_alg: Algorithm::ED25519,
            delta: 50,
            id: 0,
            net_map: HashMap::new(),
            num_faults: 0,
            num_nodes: 0,
            pk_map: HashMap::new(),
            secret_key_bytes: Vec::new(),
            payload: 0,
            bi_pp_map: HashMap::new(),
            bi_p: None,
            rand_beacon_parameter: None,
            rand_beacon_queue: HashMap::new(),
            rand_beacon_shares: Vec::new(),
        }
    }

    pub fn from_json(filename: String) -> Node {
        let f = File::open(filename).unwrap();
        let c: Node = from_reader(f).unwrap();
        return c;
    }

    pub fn from_toml(filename: String) -> Node {
        let mut buf = String::new();
        let mut f = File::open(filename).unwrap();
        f.read_to_string(&mut buf).unwrap();
        let c: Node = from_str(&buf).unwrap();
        return c;
    }

    pub fn from_yaml(filename: String) -> Node {
        let f = File::open(filename).unwrap();
        let c: Node = serde_yaml::from_reader(f).unwrap();
        return c;
    }

    pub fn from_bin(filename: String) -> Node {
        let mut buf = Vec::new();
        let mut f = File::open(filename).unwrap();
        f.read_to_end(&mut buf).unwrap();
        let bytes: &[u8] = &buf;
        let c: Node = bincode::deserialize(bytes).unwrap();
        return c;
    }

    pub fn update_config(&mut self, ips: Vec<String>) {
        let mut idx = 0;
        for ip in ips {
            // For self ip, put 0.0.0.0 with the same port
            if idx == self.id {
                let port: u16 = ip
                    .split(":")
                    .last()
                    .expect("invalid ip found; unable to split at :")
                    .parse()
                    .expect("failed to parse the port after :");
                self.net_map.insert(idx, format!("0.0.0.0:{}", port));
                idx += 1;
                continue;
            }
            // Put others ips in the config
            self.net_map.insert(idx, ip);
            idx += 1;
        }
        // println!("Talking to servers: {:?}", self.net_map);
    }

    pub fn my_ip(&self) -> String {
        // Small string, so it is okay to clone
        self.net_map
            .get(&self.id)
            .expect("Failed to obtain IP for self. Incorrect config file.")
            .clone()
    }

    /// Returns the address at which a server should listen to incoming client
    /// connections
    pub fn client_ip(&self) -> String {
        format!("0.0.0.0:{}", self.client_port)
    }
}
