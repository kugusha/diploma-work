# Ring Signature
The given app provides blockchain solution with ring signature. 
The project contains five essential files and was build according to 0.9.0 version of Exonum framework.
 
## Transactions
The project allows to use five transactions: Vote, CreateWallet, RingSignature, AddCandidate, SetVoterList.
The first one is about voting. To deploy this transaction public keys of sender and receiver are required. 

The second is CreateWallet. "Standard" transaction of creating wallet, which requires public key of the holder and wallet name.

Ring Signature transaction creates cryptographic signature using public and secret keys of the deployer and public keys from voters list.

The others two types are about setting list of voters and candidates into the blockchain.

## Wallet
Every transaction is run with the given entity named wallet. Wallet structure has five field: history, length of history, holder's public key, balance and name. The balance is equal to zero.

## Schema and Api
Schema contains all the information about wallets, candidates and voters. To see the changes in blockchain, it's neccessary to use API via postman service.

## How to build
The first step you need to open Cargo.toml file in ``` exonum/examples/ring-signature/backend ``` directory and add change ```name``` in ```[package]``` section to "exonum-ring-signature"

Then go to ```exonum/ ``` directory and add "examples/ring-signature/backend" to ```members``` array

The third step is to open main.rs file and insert the code below:
```sh
extern crate exonum;
extern crate exonum_configuration;
extern crate exonum_ring_signature;
extern crate exonum_time;

use exonum::helpers::{self, fabric::NodeBuilder};
use exonum_configuration as configuration;
use exonum_ring_signature as cryptocurrency;
use exonum_time::TimeServiceFactory;


fn main() {
    exonum::crypto::init();
    helpers::init_logger().unwrap();

    let node = NodeBuilder::new()
        .with_service(Box::new(configuration::ServiceFactory))
        .with_service(Box::new(TimeServiceFactory))
        .with_service(Box::new(cryptocurrency::ServiceFactory));
    node.run();
}
```
The step number 4: go to ``` exonum/examples/ring-signature/backend ``` directory and run comand via console 

``` cargo  build```

## How to run

Step 1

From ``` target/debug ``` directory move "exonum-ring-signature" file to``` exonum/examples/ring-signature/backend ``` directory

Step 2

Set validators:
```sh
./exonum-ring-signature generate-template example/common.toml --validators-count 4
```

Generate public and secrets keys for each node:

```sh
./exonum-ring-signature generate-config example/common.toml  example/pub_1.toml example/sec_1.toml --peer-address 127.0.0.1:6331

./exonum-ring-signature generate-config example/common.toml  example/pub_2.toml example/sec_2.toml --peer-address 127.0.0.1:6332

./exonum-ring-signature generate-config example/common.toml  example/pub_3.toml example/sec_3.toml --peer-address 127.0.0.1:6333

./exonum-ring-signature generate-config example/common.toml  example/pub_4.toml example/sec_4.toml --peer-address 127.0.0.1:6334
```

Finalize configs:

```sh
./exonum-ring-signature finalize --public-api-address 0.0.0.0:8200 --private-api-address 0.0.0.0:8091 example/sec_1.toml example/node_1_cfg.toml --public-configs example/pub_1.toml example/pub_2.toml example/pub_3.toml example/pub_4.toml

./exonum-ring-signature finalize --public-api-address 0.0.0.0:8201 --private-api-address 0.0.0.0:8092 example/sec_2.toml example/node_2_cfg.toml --public-configs example/pub_1.toml example/pub_2.toml example/pub_3.toml example/pub_4.toml

./exonum-ring-signature finalize --public-api-address 0.0.0.0:8202 --private-api-address 0.0.0.0:8093 example/sec_3.toml example/node_3_cfg.toml --public-configs example/pub_1.toml example/pub_2.toml example/pub_3.toml example/pub_4.toml

./exonum-ring-signature finalize --public-api-address 0.0.0.0:8203 --private-api-address 0.0.0.0:8094 example/sec_4.toml example/node_4_cfg.toml --public-configs example/pub_1.toml example/pub_2.toml example/pub_3.toml example/pub_4.toml
```

Run nodes:

```sh
./exonum-ring-signature run --node-config example/node_1_cfg.toml --db-path example/db1 --public-api-address 0.0.0.0:8200

./exonum-ring-signature run --node-config example/node_2_cfg.toml --db-path example/db2 --public-api-address 0.0.0.0:8201

./exonum-ring-signature run --node-config example/node_3_cfg.toml --db-path example/db3 --public-api-address 0.0.0.0:8202

./exonum-ring-signature run --node-config example/node_4_cfg.toml --db-path example/db4 --public-api-address 0.0.0.0:8203
```

