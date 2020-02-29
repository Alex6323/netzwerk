# netzwerk

`netzwerk` is a library, that lets you build peer-to-peer (p2p) nodes, and which is:

* async (based on async-std),
* event-driven,
* lock-free and free of globally shared state (instead: message passing, actor model),
* easily extendable by adding commands, events, network protocols

## Supported protocols
* TCP
* UDP (wip)

## Run the example

Open two terminals and run the following two commands respectively.

```Rust
cargo r --example node -- --bind localhost:1337 --peers tcp://localhost:1338 --msg ping
cargo r --example node -- --bind localhost:1338 --peers tcp://localhost:1337 --msg pong

```

## Disclaimer

Do not use this crate for any serious application. This is just a prototype for another project.