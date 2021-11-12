## ReentrancyDemo Usage

### 1. Setting up the test environment in NEAR's LocalNet (Sandbox)

Clone the `nearcore` repo:

```bash
git clone https://github.com/near/nearcore
cd nearcore
```

Build the sandbox binary which will take several minutes depending on your CPU:

```bash
make sandbox
```

### 2. Start and Stop Sandbox Node

Start the sandbox node:

```bash
target/debug/near-sandbox --home /tmp/near-sandbox init
target/debug/near-sandbox --home /tmp/near-sandbox run
```

Once you're finished using the sandbox node you can stop it by using `Ctrl-C`. To clean up the data it generates, simply run:

```bash
rm -rf /tmp/near-sandbox
```

### 3. Run the Demo Test in Sandbox

Start a NEAR Sandbox node. If you've already run a sandbox node with tests make sure you delete `/tmp/near-sandbox` before restarting the node.

The test script, located at ReentrancyDemo/attack_contract/Triple_Contracts_Reentrancy.js, automates the deployment of contracts in the local sandbox and creates simulated users for cross-contract calls.

Let's do some preparation for the test:

```sh
cd attack_contract
npm init
npm i near-api-js bn.js
```

and then compile each smart contract into Wasm binaries using the following command line:

```sh
RUSTFLAGS='-C link-arg=-s' cargo +stable build --target wasm32-unknown-unknown --release
```

Run the script to execute our reentrancy demo:

```sh
cd attack_contract
node Triple_Contracts_Reentrancy.js
```

