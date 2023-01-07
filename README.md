# open-defender-rust

* Contains the 3 servers that is currently written in Rust: `tss_share_2_server`, `tss_client_server` and `tss_sm_manager`
* The ZenGo library `multi-party-ecdsa` is referred to as submodule
* `tss_sm_signing` is used as a functional library, no main function. It's used by `share_2_server` and `client_server`
* The command to start each server is put below

## tss architecture
![image](https://user-images.githubusercontent.com/23033847/210502433-785f4faf-8e85-4403-9163-507c17137ee1.png)

## tss_client_server

```bash
cargo run -p tss_client_server
```

## tss_share_2_server

```bash
cargo run -p tss_share_2_server
```

## tss_sm_manager

```bash
cargo run -p tss_sm_manager
```
