# Mansion

Check out `examples/client.rs` and `examples/server.rs` for basic usage.
```bash
# Run client example
cargo r --example client --features client

# Run server example
cargo r --example server --features server

# Encryption example
# Take a look at `examples/enc_client.rs` and `examples/enc_server.rs` for using encryption.
# You need to generate keys first. To do it run example `enc_gen`
cargo r --example enc_gen

# Run encryption client
cargo r --example enc_client --features client,encrypted

# Run encryption server
cargo r --example enc_server --features server,encrypted
```