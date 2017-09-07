//! See the type-level documentation for [`WsTransport`](struct.WsTransport.html).

#![cfg_attr(feature = "strict", deny(warnings))]
#![cfg_attr(feature = "strict", deny(missing_docs, missing_debug_implementations))]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(doc_markdown))]
#![doc(html_root_url = "https://docs.rs/websocket-transport/0.1.0")]

#[macro_use]
extern crate futures;
extern crate websocket;

use futures::{Async, AsyncSink, Poll, Sink, StartSend, Stream};
use std::fmt;
use std::str::Utf8Error;
use websocket::message::OwnedMessage;

/// An easy wrapper around an async WebSocket which implements
/// [`Stream`](https://docs.rs/futures/0.1.15/futures/stream/trait.Stream.html)
/// and
/// [`Sink`](https://docs.rs/futures/0.1.15/futures/sink/trait.Sink.html)
/// for `String`.
///
/// This type automatically takes care of:
///
/// - receiving and responding to `Ping`s, as the `Stream` is polled
/// - attempting to convert `Binary` messages to UTF-8 `String`s
///
/// It can be wrapped around
/// [`Client`](https://docs.rs/websocket/0.20.2/websocket/client/async/type.Client.html)
/// or any other type which implements
/// [`Stream`](https://docs.rs/futures/0.1.15/futures/stream/trait.Stream.html)
/// and
/// [`Sink`](https://docs.rs/futures/0.1.15/futures/sink/trait.Sink.html) for
/// [`OwnedMessage`](https://docs.rs/websocket/0.20.2/websocket/message/enum.OwnedMessage.html).
pub struct WsTransport<T> {
    inner: T,
    sending: Option<OwnedMessage>,
    flushing: bool,
}

impl<T> WsTransport<T> {
    /// Wrap around an inner async WebSocket transport.
    ///
    /// `T` can be
    /// [`Client`](https://docs.rs/websocket/0.20.2/websocket/client/async/type.Client.html)
    /// or any other type which implements
    /// [`Stream`](https://docs.rs/futures/0.1.15/futures/stream/trait.Stream.html)
    /// and
    /// [`Sink`](https://docs.rs/futures/0.1.15/futures/sink/trait.Sink.html) for
    /// [`OwnedMessage`](https://docs.rs/websocket/0.20.2/websocket/message/enum.OwnedMessage.html).
    pub fn new(inner: T) -> Self {
        WsTransport {
            inner: inner,
            sending: None,
            flushing: false,
        }
    }
}

impl<T> fmt::Debug for WsTransport<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WsTransport")
            .field("ws", &Omitted)
            .field("sending", &self.sending)
            .field("flushing", &self.flushing)
            .finish()
    }
}

impl<T> Stream for WsTransport<T>
where
    T: Stream<Item = OwnedMessage>,
    T: Sink<SinkItem = OwnedMessage, SinkError = <T as Stream>::Error>,
    T::Error: From<Utf8Error>,
{
    type Item = String;
    type Error = T::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            if let Some(msg) = self.sending.take() {
                if let AsyncSink::NotReady(msg) = self.inner.start_send(msg)? {
                    self.sending = Some(msg);
                    return Ok(Async::NotReady);
                }
                self.flushing = true;
            }

            if self.flushing {
                try_ready!(self.inner.poll_complete());
                self.flushing = false;
            }

            let item = try_ready!(self.inner.poll());
            match item {
                None => return Ok(None.into()),
                Some(OwnedMessage::Text(text)) => {
                    return Ok(Some(text).into());
                }
                Some(OwnedMessage::Binary(bytes)) => {
                    let text = String::from_utf8(bytes).map_err(|err| err.utf8_error().into())?;
                    return Ok(Some(text).into());
                }
                Some(OwnedMessage::Ping(data)) => {
                    self.sending = Some(OwnedMessage::Pong(data));
                }
                Some(OwnedMessage::Close(_)) | Some(OwnedMessage::Pong(_)) => (),
            }
        }
    }
}

impl<T> Sink for WsTransport<T>
where
    T: Sink<SinkItem = OwnedMessage>,
{
    type SinkItem = String;
    type SinkError = T::SinkError;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        Ok(match self.inner.start_send(OwnedMessage::Text(item))? {
            AsyncSink::Ready => AsyncSink::Ready,
            AsyncSink::NotReady(msg) => AsyncSink::NotReady(match msg {
                OwnedMessage::Text(item) => item,
                _ => unreachable!("websocket-transport: inner Sink broke its contract"),
            }),
        })
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        self.inner.poll_complete()
    }

    fn close(&mut self) -> Poll<(), Self::SinkError> {
        self.inner.close()
    }
}

struct Omitted;

impl fmt::Debug for Omitted {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "...")
    }
}
