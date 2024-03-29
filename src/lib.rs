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

//! Cryptocurrency implementation example using [exonum](http://exonum.com/).

#![deny(missing_debug_implementations, unsafe_code, bare_trait_objects)]

#[macro_use]
extern crate exonum;
extern crate exonum_time;
#[macro_use]
extern crate failure;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate chrono;

pub use schema::CurrencySchema;

pub mod api;
pub mod schema;
pub mod transactions;
pub mod wallet;

use exonum::{
    api::ServiceApiBuilder, blockchain::{Service, Transaction, TransactionSet}, crypto::Hash,
    encoding::Error as EncodingError, helpers::fabric::{self, Context}, messages::RawTransaction,
    storage::Snapshot,
};

use transactions::WalletTransactions;

/// Unique service ID.
const RING_SIGNATURE_ID: u16 = 128;
/// Name of the service.
pub const SERVICE_NAME: &str = "cryptocurrency";
/// Initial balance of the wallet.
const INITIAL_BALANCE: u64 = 1;

/// Exonum `Service` implementation.
#[derive(Default, Debug)]
pub struct CurrencyService;

impl Service for CurrencyService {
    fn service_name(&self) -> &str {
        SERVICE_NAME
    }

    fn service_id(&self) -> u16 {
        RING_SIGNATURE_ID
    }

    fn state_hash(&self, view: &dyn Snapshot) -> Vec<Hash> {
        let schema = CurrencySchema::new(view);
        schema.state_hash()
    }

    fn tx_from_raw(&self, raw: RawTransaction) -> Result<Box<dyn Transaction>, EncodingError> {
        WalletTransactions::tx_from_raw(raw).map(Into::into)
    }

    fn wire_api(&self, builder: &mut ServiceApiBuilder) {
        api::CryptocurrencyApi::wire(builder);
    }
}

#[derive(Debug)]
pub struct ServiceFactory;

impl fabric::ServiceFactory for ServiceFactory {
    fn service_name(&self) -> &str {
        SERVICE_NAME
    }

    fn make_service(&mut self, _: &Context) -> Box<dyn Service> {
        Box::new(CurrencyService)
    }
}
