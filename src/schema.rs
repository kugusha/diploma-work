// Copyright 2018 The Exonum Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.  

use exonum::{
    crypto::{Hash, PublicKey, SecretKey, Signature}, storage::{Fork, ProofListIndex, ProofMapIndex, Snapshot, MapIndex},
    messages::{RawMessage},
};

use exonum::crypto;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use wallet::Wallet;
use INITIAL_BALANCE;


encoding_struct! {
    /// Timestamp entry.
    struct TimestampEntry {

        /// Hash of transaction.
        tx_hash: &Hash,

        /// Timestamp time.
        time: DateTime<Utc>,
    }

}



/// Database schema for the cryptocurrency.
#[derive(Debug)]
pub struct CurrencySchema<T> {
    view: T,
}

impl<T> AsMut<T> for CurrencySchema<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.view
    }
}

impl<T> CurrencySchema<T>
where
    T: AsRef<dyn Snapshot>,
{
    /// Constructs schema from the database view.
    pub fn new(view: T) -> Self {
        CurrencySchema { view }
    }

    /// Returns `MerklePatriciaTable` with wallets.
    pub fn wallets(&self) -> ProofMapIndex<&T, PublicKey, Wallet> {
        ProofMapIndex::new("cryptocurrency.wallets", &self.view)
    }

    /// Returns history of the wallet with the given public key.
    pub fn wallet_history(&self, public_key: &PublicKey) -> ProofListIndex<&T, Hash> {
        ProofListIndex::new_in_family("cryptocurrency.wallet_history", public_key, &self.view)
    }

    /// Returns wallet for the given public key.
    pub fn wallet(&self, pub_key: &PublicKey) -> Option<Wallet> {
        self.wallets().get(pub_key)
    }

    /// Returns state hash of service database.
    pub fn state_hash(&self) -> Vec<Hash> {
        vec![self.wallets().merkle_root()]
    }

    /// Returns table that represents a map from transaction hash into raw transaction message.
    pub fn transactions(&self) -> MapIndex<&T, Hash, RawMessage> {
        MapIndex::new("core.transactions", &self.view)
    }

    pub fn voters_list(&self) -> ProofMapIndex<&T, PublicKey, PublicKey> {
        ProofMapIndex::new("cryptocurrency.voters", &self.view)
    }

    pub fn candidates_list(&self) -> ProofMapIndex<&T, PublicKey, PublicKey> {
        ProofMapIndex::new("cryptocurrency.candidates", &self.view)
    }

    /// Returns the `ProofMapIndex` of timestamps.
    pub fn timestamps(&self) -> ProofMapIndex<&T, Hash, i64> {
        ProofMapIndex::new("cryptocurrency.timestamps", &self.view)
    }

    /// Returns the state hash of the timestamping service.
    pub fn state_hash_timestamps(&self) -> Vec<Hash> {
        vec![self.timestamps().merkle_root()]
    }

    pub fn state_hash_voters_list(&self) -> Vec<Hash> {
        vec![self.voters_list().merkle_root()]
    }
}

/// Implementation of mutable methods.
impl<'a> CurrencySchema<&'a mut Fork> {
    /// Returns mutable `MerklePatriciaTable` with wallets.
    pub fn wallets_mut(&mut self) -> ProofMapIndex<&mut Fork, PublicKey, Wallet> {
        ProofMapIndex::new("cryptocurrency.wallets", &mut self.view)
    }

    /// Returns history for the wallet by the given public key.
    pub fn wallet_history_mut(
        &mut self,
        public_key: &PublicKey,
    ) -> ProofListIndex<&mut Fork, Hash> {
        ProofListIndex::new_in_family("cryptocurrency.wallet_history", public_key, &mut self.view)
    }

    /// Increase balance of the wallet and append new record to its history.
    ///
    /// Panics if there is no wallet with given public key.
    //pub fn increase_wallet_balance(&mut self, wallet: Wallet, amount: u64, transaction: &Hash, freezed_balance: u64) {
    pub fn increase_wallet_balance(&mut self, wallet: Wallet, amount: u64, transaction: &Hash) {
        let wallet = {
            let mut history = self.wallet_history_mut(wallet.pub_key());
            history.push(*transaction);
            let history_hash = history.merkle_root();
            let balance = wallet.balance();
            /////////////////////////////
            //wallet.set_balance(balance + amount, &history_hash, freezed_balance)
            wallet.set_balance(balance + amount, &history_hash)
        };
        self.wallets_mut().put(wallet.pub_key(), wallet.clone());
    }

    /// Decrease balance of the wallet and append new record to its history.
    ///
    /// Panics if there is no wallet with given public key.
    pub fn decrease_wallet_balance(&mut self, wallet: Wallet, amount: u64, transaction: &Hash) {
        let wallet = {
            let mut history = self.wallet_history_mut(wallet.pub_key());
            history.push(*transaction);
            let history_hash = history.merkle_root();
            let balance = wallet.balance();
            wallet.set_balance(balance - amount, &history_hash)
        };
        self.wallets_mut().put(wallet.pub_key(), wallet.clone());
    }

    /// Create new wallet and append first record to its history.
    //pub fn create_wallet(&mut self, key: &PublicKey, name: &str, transaction: &Hash, freezed_balance: u64) {
    pub fn create_wallet(&mut self, key: &PublicKey, name: &str, transaction: &Hash) {
        let wallet = {
            let mut history = self.wallet_history_mut(key);
            history.push(*transaction);
            let history_hash = history.merkle_root();
            //let freezed_balance = 0;
            //Wallet::new(key, name, INITIAL_BALANCE, history.len(), &history_hash, freezed_balance)
            Wallet::new(key, name, INITIAL_BALANCE, history.len(), &history_hash)
        };
        self.wallets_mut().put(key, wallet);
    }

    /// Returns mut table that represents a map from transaction hash into raw transaction message.
    pub fn transactions_mut(&mut self) -> MapIndex<&mut Fork, Hash, RawMessage> {
        MapIndex::new("core.transactions", &mut self.view)
    }

    /// Returns the mutable `ProofMapIndex` of timestamps.
    pub fn timestamps_mut(&mut self) -> ProofMapIndex<&mut Fork, Hash, i64> {
        ProofMapIndex::new("cryptocurrency.timestamps", &mut self.view)
    }

    /// Adds the timestamp entry to the database.
    pub fn add_timestamp(&mut self, timestamp_entry: TimestampEntry) {
        let tx_hash = timestamp_entry.tx_hash();
        let time = timestamp_entry.time();

        // Check that timestamp with given content_hash does not exist.
        if self.timestamps().contains(tx_hash) {
            return;
        }
        // Add timestamp
        self.timestamps_mut().put(tx_hash, time.timestamp());
    }

    pub fn voters_list_mut(&mut self) -> ProofMapIndex<&mut Fork, PublicKey, PublicKey> {
        ProofMapIndex::new("cryptocurrency.voters", &mut self.view)
    }

    pub fn append_voter(&mut self, key: &PublicKey, sec_key: &PublicKey) {
        let pk = key;
        let sk = sec_key;
        self.voters_list_mut().put(pk, *sk);
    }

    pub fn candidates_list_mut(&mut self) -> ProofMapIndex<&mut Fork, PublicKey, PublicKey> {
        ProofMapIndex::new("cryptocurrency.candidates", &mut self.view)
    }

    pub fn append_candidate(&mut self, key: &PublicKey, sec_key: &PublicKey) {
        let pk = key;
        let sk = sec_key;
        self.candidates_list_mut().put(pk, *sk);
    }

}