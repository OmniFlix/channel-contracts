# **Raffle Contract**

# **Overview**

This contract manages the raffle for the LAB Drop involving 10,028 NFT raffle tickets minted on the OmniFlix Hub by the StreamSwap DAO. This contract has been developed by OmniFlix to ensure a fair and random raffle process is offered to creators and communities.

The contract is deployed on the Juno Network and integrates with the Nois protocol to ensure fair and transparent selection of random winners.

# **Functionality**

- **Instantiation**: Sets up the contract with the Nois proxy address and participant count.
- **Execution**:
    - **Request Randomness**: Initiates a randomness request to the Nois protocol, requiring a unique job ID and necessary funds.
    - **Nois Receive**: Receives randomness from Nois, then saves the randomness based on the job ID. If the job id constains the word "test", the contract will save the randomness as a test value. Otherwise, it will save the randomness as the final value. Test value can be reseted if another request is made. But the final value can only be set once.
    - **Pick Test Winners**: Selects test winners based on the test randomness value.
    - **Pick Winners**: Selects winners based on the final randomness value.
- **Queries**: Provides smart queries for accessing participant count, list of winners, admin details, etc.

# **Setup and Configuration**

### **Prerequisites**

- Node.js and Rust installed.
- Access to a Juno Network node or testnet.

### **Installation**

Clone the repository and build the project:

```bash
git clone [repository-url]

cd raffle-drop-main

cargo build --release

```

### Optimize

```
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.13.0
```

### **Deployment**

Deploy and instantiate on the Juno Network:

```bash
junod tx wasm store artifacts/contract.wasm --from <your-wallet> --chain-id <chain-id> --gas auto --fees <fee-amount>

junod tx wasm instantiate <code-id> '{"nois_proxy_address": "<address>", "participant_count": 10028}' --label "lab_drop_raffle" --from <your-wallet> --chain-id <chain-id> --amount <init-amount> --gas auto --fees <fee-amount>
```

### **Interaction**

Execute transactions and query contract state:

```bash
junod tx wasm execute <contract-address> '{"RequestRandomness": {"job_id": "unique_job_id"}}' --from <your-wallet> --chain-id <chain-id> --gas auto --fees <fee-amount>

junod query wasm contract-state smart <contract-address> '{"QueryMsg": "Winners"}'
```

# **Contributing**

For use under the MIT license, all contributions are welcome. Ensure all modifications pass existing tests and new tests are added for novel functionality.
