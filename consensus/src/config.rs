use crypto::PublicKey;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::convert::TryInto;

pub type Index = u32;
pub type Stake = u32;
pub type EpochNumber = u128;

#[derive(Serialize, Deserialize)]
pub struct Parameters {
    pub timeout_delay: u64,
    pub sync_retry_delay: u64,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            timeout_delay: 5_000,
            sync_retry_delay: 10_000,
        }
    }
}

impl Parameters {
    pub fn log(&self) {
        // NOTE: These log entries are used to compute performance.
        info!("Timeout delay set to {} rounds", self.timeout_delay);
        info!("Sync retry delay set to {} ms", self.sync_retry_delay);
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Authority {
    pub index: Index,
    pub stake: Stake,
    pub address: SocketAddr,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Committee {
    pub authorities: HashMap<PublicKey, Authority>,
    pub epoch: EpochNumber,
}

impl Committee {
    pub fn new(info: Vec<(PublicKey, Index, Stake, SocketAddr)>, epoch: EpochNumber) -> Self {
        Self {
            authorities: info
                .into_iter()
                .map(|(name, index, stake, address)| {
                    let authority = Authority { index, stake, address };
                    (name, authority)
                })
                .collect(),
            epoch,
        }
    }

    pub fn size(&self) -> usize {
        self.authorities.len()
    }

    pub fn stake(&self, name: &PublicKey) -> Stake {
        self.authorities.get(name).map_or_else(|| 0, |x| x.stake)
    }

    pub fn quorum_threshold(&self) -> Stake {
        // If N = 3f + 1 + k (0 <= k < 3)
        // then (2 N + 3) / 3 = 2f + 1 + (2k + 2)/3 = 2f + 1 + k = N - f
        let total_votes: Stake = self.authorities.values().map(|x| x.stake).sum();
        2 * total_votes / 3 + 1
    }

    pub fn address(&self, name: &PublicKey) -> Option<SocketAddr> {
        self.authorities.get(name).map(|x| x.address)
    }

    pub fn broadcast_addresses(&self, myself: &PublicKey) -> Vec<(PublicKey, SocketAddr)> {
        self.authorities
            .iter()
            .filter(|(name, _)| name != &myself)
            .map(|(name, x)| (*name, x.address))
            .collect()
    }

    pub fn all_public_keys(&self) -> Vec<PublicKey> {
        self.authorities
            .iter()
            .map(|(name, _)| *name)
            .collect()
    }

    pub fn public_keys_to_binary_repr(&self, pks: &Vec<PublicKey>) -> Vec<bool> {
        let mut binary = vec![false; self.authorities.len()];
        for pk in pks{
            let i: usize = self.authorities.get(pk).unwrap().index.try_into().unwrap();
            binary[i] = true;
        }
        return binary;
    }

    pub fn binary_repr_to_public_keys(&self, binary_repr: &Vec<bool>) -> Vec<PublicKey> {
        let keys: HashMap<Index, PublicKey> = self.authorities.keys().into_iter().map(|pk| (self.authorities.get(pk).unwrap().index, pk.clone())).collect();
        let mut pks: Vec<PublicKey> = Vec::new();
        for i in 0..binary_repr.len(){
            if binary_repr[i] == true {
                let index = i.try_into().unwrap(); 
                pks.push(*keys.get(&index).unwrap());
            }
        }
        return pks;
    }
}
