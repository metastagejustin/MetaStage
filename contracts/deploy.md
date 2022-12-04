To deploy the contract on mainnet, one has to perform the following actions:

1. Install Rust on your machine (see Rust documentation);
2. Install wasm-pack on your machine, this can be achieved by executing

  -- rustup target add wasm32-unknown-unknown

3. Build the target wasm compiled code by executing

  -- ./build.sh

4. Create a Near mainnet account and deposit some NEAR tokens on it
(around 2.5 NEAR tokens should be sufficient)
5. Install the near-cli, by executing

  -- npm install i near-cli

6. Set near env variable to mainnet (otherwise it will default to testnet)

  -- export NEAR_ENV=mainnet

7. Login into your Near account, via the cli

  -- near login

and follow the instructions.