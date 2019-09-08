
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

#![ allow( bare_trait_objects ) ]

extern crate serde_json;
extern crate serde;

use serde::{Deserialize, Serialize, Deserializer, Serializer};

use chrono::{DateTime, Utc};

use exonum::blockchain::{ExecutionError, ExecutionResult, Transaction};
use exonum::crypto::{CryptoHash, PublicKey, Hash, Signature, SecretKey};
use exonum::messages::Message;
use exonum::storage::Fork;
use exonum::storage::StorageValue;
use exonum::messages::RawMessage;
use exonum::storage::Snapshot;
use exonum_time::schema::TimeSchema;
use exonum :: crypto;

use RING_SIGNATURE_ID;
use schema::{CurrencySchema, TimestampEntry};

/// Error codes emitted by wallet transactions during execution.
#[derive(Debug, Fail)]
#[repr(u8)]
pub enum Error {
    /// Wallet already exists.
    ///
    /// Can be emitted by `CreateWallet`.
    #[fail(display = "Wallet already exists")]
    WalletAlreadyExists = 0,

    /// Sender doesn't exist.
    ///
    /// Can be emitted by `Transfer`.
    #[fail(display = "Sender doesn't exist")]
    SenderNotFound = 1,

    /// Receiver doesn't exist.
    ///
    /// Can be emitted by `Transfer` or `Issue`.
    #[fail(display = "Receiver doesn't exist")]
    ReceiverNotFound = 2,

    /// Insufficient currency amount.
    ///
    /// Can be emitted by `Transfer`.
    #[fail(display = "Insufficient currency amount")]
    InsufficientCurrencyAmount = 3,

    #[fail(display = "Time is up")]
    Timeisup = 4,

}

impl From<Error> for ExecutionError {
    fn from(value: Error) -> ExecutionError {
        let description = format!("{}", value);
        ExecutionError::with_description(value as u8, description)
    }
}

transactions! {
    pub WalletTransactions {
        const SERVICE_ID = RING_SIGNATURE_ID;

        /// Transfer `amount` of the currency from one wallet to another.
        struct Vote {
            from:    &PublicKey,
            to:      &PublicKey,
            amount:  u64,
            seed:    u64,
        }

        /// Create wallet with the given `name`.
        struct CreateWallet {
            pub_key: &PublicKey,
            name:    &str,
        }

        /// Ring Signature Tx
        struct RingSignature{
            from: &PublicKey,
            seed: u64,
        }

        ///Add candidate
        struct AddCandidate{
            pub_key: &PublicKey,
            seed: u64,
        }

        ///Add voter
        struct SetVoterList{
            pub_key: &PublicKey,
            seed: u64,
        }
    }
}

impl Transaction for Vote {
    fn verify(&self) -> bool {
        (self.from() != self.to()) && self.verify_signature(self.from())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let time = TimeSchema::new(&fork)
            .time()
            .get();
        
        let mut schema = CurrencySchema::new(fork);
        let from = self.from();
        let to = self.to();
        let hash = self.hash();
        let amount = self.amount();
        let freezed_balance = 0;

        let sender = schema.wallet(from).ok_or(Error :: SenderNotFound)?;
        let receiver = schema.wallet(to).ok_or(Error :: ReceiverNotFound)?;

        if sender.balance() < amount {
            Err(Error::InsufficientCurrencyAmount)?;

        }

        schema.decrease_wallet_balance(sender, amount, &hash);
        schema.increase_wallet_balance(receiver, amount, &hash);
        
        let entry = TimestampEntry::new(&self.hash(), time.unwrap());
        schema.add_timestamp(entry);
        Ok(())
    }
}

impl Transaction for CreateWallet {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let time = TimeSchema::new(&fork)
            .time()
            .get();
        let mut schema = CurrencySchema::new(fork);
        let pub_key = self.pub_key();
        let hash = self.hash();
        if schema.wallet(pub_key).is_none(){
            let name = self.name();
            schema.create_wallet(pub_key, name, &hash);

            let entry = TimestampEntry::new(&self.hash(), time.unwrap());
            schema.add_timestamp(entry);
            let wallet = schema.wallet(pub_key).ok_or(Error :: SenderNotFound)?;
            Ok(())
        } else {
            Err(Error::WalletAlreadyExists)?
        } 
    }    
}

impl Transaction for RingSignature{
    fn verify(&self) -> bool {
        self.verify_signature(self.from())        
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let mut schema = CurrencySchema :: new(fork);
        let (pub_key, sec_key) = crypto::gen_keypair();
        let mut posible_candidates: [u8; 1023] = [0; 1023];
        let mut count: u8 = 0;
        let mut index = 0;
        let mut voters = Vec::new();
        let mut pair_pubkey_wallet = schema.voters_list_mut();
        for i in pair_pubkey_wallet.iter() {
            voters.push(i.1);
            posible_candidates[index] = count;
            index += 1;
            count += 1;
        }
        let data = &posible_candidates[..voters.len()];
        let signature = crypto :: sign(&data, &sec_key);

        Ok(())
    }
}

impl Transaction for AddCandidate{
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())        
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult{
        let mut schema = CurrencySchema :: new(fork);
        let pub_key = self.pub_key();
        schema.append_candidate(pub_key, pub_key);
        Ok(())
    }
}

impl Transaction for SetVoterList{
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())        
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult{
        let mut schema = CurrencySchema :: new(fork);
        let pub_key = self.pub_key();
        schema.append_voter(pub_key, pub_key);
        Ok(())
    }
}