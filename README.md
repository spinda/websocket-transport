# websocket-transport

> Easy async WebSocket wrapper which implements
> [`Stream`](https://docs.rs/futures/0.1.15/futures/stream/trait.Stream.html)
> and
> [`Sink`](https://docs.rs/futures/0.1.15/futures/sink/trait.Sink.html)
> for `String`

[![Crates.io](https://img.shields.io/crates/v/websocket-transport.svg)](https://crates.io/crates/websocket-transport)
[![Linux/OSX Build Status](https://img.shields.io/travis/spinda/websocket-transport/master.svg)](https://travis-ci.org/spinda/websocket-transport)
[![Windows Build Status](https://img.shields.io/appveyor/ci/spinda/websocket-transport/master.svg)](https://ci.appveyor.com/project/spinda/websocket-transport)

[Documentation](https://docs.rs/websocket-transport/0.1.0)

## Usage

First, add this to your `Cargo.toml`:

```toml
[dependencies]
websocket-transport = "0.1"
```

Next, add this to your crate:

```rust
extern crate websocket_transport;

use websocket_transport::WsTransport;
```

## Overview

The
[`WsTransport`](https://docs.rs/websocket-transport/0.1.0/websocket_transport/struct.WsTransport.html)
type provides an easy wrapper around an async WebSocket which implements
[`Stream`](https://docs.rs/futures/0.1.15/futures/stream/trait.Stream.html)
and
[`Sink`](https://docs.rs/futures/0.1.15/futures/sink/trait.Sink.html)
for `String`.

This type automatically takes care of:

- receiving and responding to `Ping`s, as the `Stream` is polled
- attempting to convert `Binary` messages to UTF-8 `String`s

It can be wrapped around
[`Client`](https://docs.rs/websocket/0.20.2/websocket/client/async/type.Client.html)
or any other type which implements
[`Stream`](https://docs.rs/futures/0.1.15/futures/stream/trait.Stream.html)
and
[`Sink`](https://docs.rs/futures/0.1.15/futures/sink/trait.Sink.html) for
[`OwnedMessage`](https://docs.rs/websocket/0.20.2/websocket/message/enum.OwnedMessage.html).

## License

Licensed under either of

 * [Apache License, Version 2.0](/LICENSE-APACHE)
 * [MIT License](/LICENSE-MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
