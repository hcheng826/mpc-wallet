# open-defender-rust

- Contains the 3 servers that is currently written in Rust: `tss_share_2_server`, `tss_client_server` and `tss_sm_manager`. The component `Tx Sender` in the following diagram is implemented in Nodejs and is maintained here: https://github.com/FDC-AI/open-defender/tree/develop/packages/tss-tx-sender
- The ZenGo library `multi-party-ecdsa` is referred to as submodule
- `tss_sm_client` is used as a functional library, no main function. It's used by `share_2_server` and `client_server`

## API documentation

https://documenter.getpostman.com/view/12538945/2s93CPqXMu

## tss architecture

### Key gen

![MPC TSS chart - key gen](https://user-images.githubusercontent.com/23033847/212588917-985a2cd5-ac7a-49eb-9529-11abcd3155f1.jpeg)


- Step 2 is done by RabbitMQ with queue name: `request-@open-defender/tss-tx-sender: share-2-server-keygen-signal` (config in `.env`)

### Sign

![MPC TSS chart - sign](https://user-images.githubusercontent.com/23033847/212588965-273394fa-d70c-447c-a173-af8f06c3db84.jpeg)

- Step 3 is done by RabbitMQ with queue name: `request-@open-defender/tss-tx-sender: share-2-server-sign-signal` (config in `.env`)

## tss_client_server

```bash
cd tss_client_server
cargo run
```

### Key gen

1. the api is `/new-key`
2. send key gen request to tx sender api `/new-key`
3. after receiving the response ack from share 2, talk to sm manager to participate key gen, and get the local-share key

### Sign

1. the api is `/send-tx`
2. send tx request to tx-sender api `/request-tx`
3. if the response says the simulation succeeded, talk to sm manager to contribute to signature

## tss_share_2_server

```bash
cd tss_share_2_server
cargo run
```

### Key gen

1. consuem the key gen signal from the queue
2. talk to sm manager to participate key gen, and save the resulting local-share key for corresponding user

### Sign

1. consume the sign signal from the queue
2. signature can be generated from sm manager
3. call tx sender api `/submit-tx` and the signature will be written into its db

## tss_sm_manager

```bash
cd tss_sm_manager
cargo run
```

### Key gen and sign

Just react to whatever requests come from client server and share 2 server
