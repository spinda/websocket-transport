#![cfg_attr(feature = "strict", deny(warnings))]
#![cfg_attr(feature = "strict", deny(missing_debug_implementations))]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(unreadable_literal))]

extern crate futures;
extern crate tokio_core;
extern crate websocket;

use futures::{Future, Sink, Stream};
use std::net::{Ipv4Addr, SocketAddr};
use tokio_core::reactor::{Core, Handle};
use websocket::ClientBuilder;
use websocket::async::Server as WsServer;
use websocket::message::OwnedMessage;

extern crate websocket_transport;

use websocket_transport::WsTransport;

fn start_server(port: u16) -> (SocketAddr, Core, Handle) {
    let core = Core::new().expect("core creation error");
    let handle = core.handle();

    // We *would* bind to port 0 and use random assignment here, but WsServer
    // doesn't support retrieving the actual bound address in async mode...
    let server_addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), port);
    let listener = WsServer::bind(server_addr, &handle).expect("server bind error");

    let serve = listener
        .incoming()
        .for_each(move |(upgrade, _remote_addr)| {
            upgrade
                .accept()
                .then(|result| {
                    let (ws, _headers) = result.expect("server http error");
                    let transport = WsTransport::new(ws);
                    let (sink, stream) = transport.split();
                    stream.forward(sink)
                })
                .then(|result| {
                    result.expect("server echo error");
                    Ok(())
                })
        })
        .then(|result| {
            result.ok().expect("server accept error");
            Ok(())
        });
    handle.spawn(serve);

    (server_addr, core, handle)
}

#[test]
fn test_text() {
    let (server_addr, mut core, handle) = start_server(13370);

    let test = ClientBuilder::new(format!("ws://{}", server_addr).as_str())
        .expect("client build error")
        .async_connect_insecure(&handle)
        .then(|result| {
            let (ws, _headers) = result.expect("client connect error");
            ws.send(OwnedMessage::Text("Hello World".into()))
        })
        .then(|result| {
            let ws = result.expect("client write error");
            ws.into_future().map_err(|(err, _websocket)| err)
        })
        .and_then(|(maybe_msg, _ws)| {
            assert_eq!(maybe_msg, Some(OwnedMessage::Text("Hello World".into())));
            Ok(())
        });
    core.run(test).expect("client read error");
}

#[test]
fn test_binary() {
    let (server_addr, mut core, handle) = start_server(13371);

    let test = ClientBuilder::new(format!("ws://{}", server_addr).as_str())
        .expect("client build error")
        .async_connect_insecure(&handle)
        .then(|result| {
            let (ws, _headers) = result.expect("client connect error");
            ws.send(OwnedMessage::Binary(b"Hello World"[..].into()))
        })
        .then(|result| {
            let ws = result.expect("client write error");
            ws.into_future().map_err(|(err, _websocket)| err)
        })
        .and_then(|(maybe_msg, _ws)| {
            assert_eq!(maybe_msg, Some(OwnedMessage::Text("Hello World".into())));
            Ok(())
        });
    core.run(test).expect("client read error");
}

#[test]
fn test_ping() {
    let (server_addr, mut core, handle) = start_server(13372);

    let test = ClientBuilder::new(format!("ws://{}", server_addr).as_str())
        .expect("client build error")
        .async_connect_insecure(&handle)
        .then(|result| {
            let (ws, _headers) = result.expect("client connect error");
            ws.send(OwnedMessage::Ping(b"Hello World"[..].into()))
        })
        .then(|result| {
            let ws = result.expect("client write error");
            ws.into_future().map_err(|(err, _websocket)| err)
        })
        .and_then(|(maybe_msg, _ws)| {
            assert_eq!(maybe_msg, Some(OwnedMessage::Pong(b"Hello World"[..].into())));
            Ok(())
        });
    core.run(test).expect("client read error");
}

#[test]
fn test_text_and_ping() {
    let (server_addr, mut core, handle) = start_server(13373);

    let test = ClientBuilder::new(format!("ws://{}", server_addr).as_str())
        .expect("client build error")
        .async_connect_insecure(&handle)
        .then(|result| {
            let (ws, _headers) = result.expect("client connect error");
            ws.send(OwnedMessage::Text("Hello World".into()))
        })
        .then(|result| {
            let ws = result.expect("client write error");
            ws.into_future().map_err(|(err, _websocket)| err)
        })
        .then(|result| {
            let (maybe_msg, ws) = result.expect("client read error");
            assert_eq!(maybe_msg, Some(OwnedMessage::Text("Hello World".into())));
            ws.send(OwnedMessage::Ping(b"Hello World"[..].into()))
        })
        .then(|result| {
            let ws = result.expect("client write error");
            ws.into_future().map_err(|(err, _websocket)| err)
        })
        .then(|result| {
            let (maybe_msg, ws) = result.expect("client read error");
            assert_eq!(maybe_msg, Some(OwnedMessage::Pong(b"Hello World"[..].into())));
            ws.send(OwnedMessage::Text("World Hello".into()))
        })
        .then(|result| {
            let ws = result.expect("client write error");
            ws.into_future().map_err(|(err, _websocket)| err)
        })
        .and_then(|(maybe_msg, _ws)| {
            assert_eq!(maybe_msg, Some(OwnedMessage::Text("World Hello".into())));
            Ok(())
        });
    core.run(test).expect("client read error");
}

#[test]
fn test_client_transport() {
    let (server_addr, mut core, handle) = start_server(13374);

    let test = ClientBuilder::new(format!("ws://{}", server_addr).as_str())
        .expect("client build error")
        .async_connect_insecure(&handle)
        .then(|result| {
            let (ws, _headers) = result.expect("client connect error");
            let transport = WsTransport::new(ws);
            transport.send("Hello World".into())
        })
        .then(|result| {
            let transport = result.expect("client write error");
            transport.into_future().map_err(|(err, _transport)| err)
        })
        .and_then(|(maybe_text, _ws)| {
            assert_eq!(maybe_text, Some("Hello World".into()));
            Ok(())
        });
    core.run(test).expect("client read error");
}
