//! This module is used to interact with the Websocket API.

mod error;
mod model;
#[cfg(test)]
mod tests;

pub use error::*;
pub use model::*;

use futures::{
    ready,
    task::{Context, Poll},
    Future, SinkExt, Stream, StreamExt,
};
use hmac_sha256::HMAC;
use serde_json::json;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{collections::VecDeque, str::FromStr};
use std::{pin::Pin, sync::Arc};
use tokio::net::TcpStream;
use tokio::time; // 1.3.0
use tokio::time::Interval;
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Stream, either plain TCP or TLS.
#[derive(Debug)]
pub enum GenericWebSocketStream {
    /// Direct socket stream.
    Plain(tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>),
    /// Proxied socket stream.
    Proxy(
        tokio_tungstenite::WebSocketStream<
            tokio_rustls::client::TlsStream<tokio_socks::tcp::Socks5Stream<TcpStream>>,
        >,
    ),
}

impl GenericWebSocketStream {
    async fn send(
        &mut self,
        msg: Message,
    ) -> std::result::Result<(), tokio_tungstenite::tungstenite::Error> {
        match self {
            GenericWebSocketStream::Plain(s) => s.send(msg).await,
            GenericWebSocketStream::Proxy(s) => s.send(msg).await,
        }
    }

    async fn next(
        &mut self,
    ) -> Option<
        std::result::Result<
            tokio_tungstenite::tungstenite::Message,
            tokio_tungstenite::tungstenite::Error,
        >,
    > {
        match self {
            GenericWebSocketStream::Plain(s) => s.next().await,
            GenericWebSocketStream::Proxy(s) => s.next().await,
        }
    }
}

pub struct Ws {
    channels: Vec<Channel>,
    stream: GenericWebSocketStream,
    buf: VecDeque<(Option<Symbol>, Data)>,
    ping_timer: Interval,
    /// Whether the websocket was opened authenticated with API keys or not
    is_authenticated: bool,
}

impl Ws {
    pub const ENDPOINT: &'static str = "ftx.com";

    async fn connect_with_endpoint(
        endpoint: &str,
        key_secret: Option<(String, String)>,
        subaccount: Option<String>,
        proxy: Option<String>,
    ) -> Result<Self> {
        let mut stream: GenericWebSocketStream = match proxy {
            Some(proxy) => {
                let socks_stream = tokio_socks::tcp::Socks5Stream::connect(
                    std::net::SocketAddr::from_str(&proxy).expect("invalid proxy addr"),
                    (Self::ENDPOINT, 443),
                )
                .await
                .expect("cannot connect to proxy");
                let mut client_config = tokio_rustls::rustls::ClientConfig::new();
                client_config
                    .root_store
                    .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
                let connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));
                let domain = webpki::DNSNameRef::try_from_ascii_str(Self::ENDPOINT).unwrap();
                let tls_stream = connector
                    .connect(domain, socks_stream)
                    .await
                    .expect("cannot create tls stream");
                let (ws_client, response) =
                    tokio_tungstenite::client_async(format!("wss://{}/ws", endpoint), tls_stream)
                        .await
                        .unwrap();
                GenericWebSocketStream::Proxy(ws_client)
            }
            None => GenericWebSocketStream::Plain(connect_async(endpoint).await?.0),
        };
        let is_authenticated = key_secret.is_some();
        if let Some((key, secret)) = key_secret {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let sign_payload = format!("{}websocket_login", timestamp);
            let sign = HMAC::mac(sign_payload.as_bytes(), secret.as_bytes());
            let sign = hex::encode(sign);

            stream
                .send(Message::Text(
                    json!({
                        "op": "login",
                        "args": {
                            "key": key,
                            "sign": sign,
                            "time": timestamp as u64,
                            "subaccount": subaccount,
                        }
                    })
                    .to_string(),
                ))
                .await?;
        }
        Ok(Self {
            channels: Vec::new(),
            stream,
            buf: VecDeque::new(),
            ping_timer: time::interval(Duration::from_secs(15)),
            is_authenticated,
        })
    }
    pub async fn connect(
        // Pair (API_KEY, SECRET_KEY) for authentification.
        // The channels FILL, ORDER, and FTX Pay require authentification
        key_secret: Option<(String, String)>,
        subaccount: Option<String>,
        proxy: Option<String>,
    ) -> Result<Self> {
        Self::connect_with_endpoint(Self::ENDPOINT, key_secret, subaccount, proxy).await
    }

    // Pair (API_KEY, SECRET_KEY) for authentification.
    // The channels FILL, ORDER, and FTX Pay require authentification
    // pub async fn connect_us(
    //     key_secret: Option<(String, String)>,
    //     subaccount: Option<String>,
    // ) -> Result<Self> {
    //     Self::connect_with_endpoint(Self::ENDPOINT_US, key_secret, subaccount).await
    // }

    async fn ping(&mut self) -> Result<()> {
        self.stream
            .send(Message::Text(
                json!({
                    "op": "ping",
                })
                .to_string(),
            ))
            .await?;

        Ok(())
    }

    /// Subscribe to specified `Channel`s
    /// For FILLS the socket needs to be authenticated
    pub async fn subscribe(&mut self, channels: Vec<Channel>) -> Result<()> {
        for channel in channels.iter() {
            // Subscribing to fills or orders requires us to be authenticated via an API key
            if (channel == &Channel::Fills || channel == &Channel::Orders) && !self.is_authenticated
            {
                return Err(Error::SocketNotAuthenticated);
            }
            self.channels.push(channel.clone());
        }

        self.subscribe_or_unsubscribe(channels, true).await?;

        Ok(())
    }

    /// Unsubscribe from specified `Channel`s
    pub async fn unsubscribe(&mut self, channels: Vec<Channel>) -> Result<()> {
        // Check that the specified channels match an existing one
        for channel in channels.iter() {
            if !self.channels.contains(channel) {
                return Err(Error::NotSubscribedToThisChannel(channel.clone()));
            }
        }

        self.subscribe_or_unsubscribe(channels.clone(), false)
            .await?;

        // Unsubscribe successful, remove specified channels from self.channels
        self.channels.retain(|c| !channels.contains(c));

        Ok(())
    }

    /// Unsubscribe from all currently subscribed `Channel`s
    pub async fn unsubscribe_all(&mut self) -> Result<()> {
        self.unsubscribe(self.channels.clone()).await?;

        self.channels.clear();

        Ok(())
    }

    async fn subscribe_or_unsubscribe(
        &mut self,
        channels: Vec<Channel>,
        subscribe: bool,
    ) -> Result<()> {
        let op = if subscribe {
            "subscribe"
        } else {
            "unsubscribe"
        };

        'channels: for channel in channels {
            let (channel, symbol) = match channel {
                Channel::Orderbook(symbol) => ("orderbook", symbol),
                Channel::Trades(symbol) => ("trades", symbol),
                Channel::Ticker(symbol) => ("ticker", symbol),
                Channel::Fills => ("fills", "".to_string()),
                Channel::Orders => ("orders", "".to_string()),
            };

            self.stream
                .send(Message::Text(
                    json!({
                        "op": op,
                        "channel": channel,
                        "market": symbol,
                    })
                    .to_string(),
                ))
                .await?;

            // Confirmation should arrive within the next 100 updates
            for _ in 0..100 {
                let response = self.next_response().await?;
                match response {
                    Response {
                        r#type: Type::Subscribed,
                        ..
                    } if subscribe => {
                        // Subscribe confirmed
                        continue 'channels;
                    }
                    Response {
                        r#type: Type::Unsubscribed,
                        ..
                    } if !subscribe => {
                        // Unsubscribe confirmed
                        continue 'channels;
                    }
                    _ => {
                        // Otherwise, continue adding contents to buffer
                        self.handle_response(response);
                    }
                }
            }

            return Err(Error::MissingSubscriptionConfirmation);
        }

        Ok(())
    }

    async fn next_response(&mut self) -> Result<Response> {
        loop {
            tokio::select! {
                _ = self.ping_timer.tick() => {
                    self.ping().await?;
                },
                Some(msg) = self.stream.next() => {
                    let msg = msg?;
                    if let Message::Text(text) = msg {
                        // println!("{}", text); // Uncomment for debugging
                        let response: Response = serde_json::from_str(&text)?;

                        // Don't return Pong responses
                        if let Response { r#type: Type::Pong, .. } = response {
                            continue;
                        }

                        return Ok(response)
                    }
                },
            }
        }
    }

    /// Helper function that takes a response and adds the contents to the buffer
    fn handle_response(&mut self, response: Response) {
        if let Some(data) = response.data {
            match data {
                ResponseData::Trades(trades) => {
                    // Trades channel returns an array of single trades.
                    // Buffer so that the user receives trades one at a time
                    for trade in trades {
                        self.buf
                            .push_back((response.market.clone(), Data::Trade(trade)));
                    }
                }
                ResponseData::OrderbookData(orderbook) => {
                    self.buf
                        .push_back((response.market, Data::OrderbookData(orderbook)));
                }
                ResponseData::Fill(fill) => {
                    self.buf.push_back((response.market, Data::Fill(fill)));
                }
                ResponseData::Ticker(ticker) => {
                    self.buf.push_back((response.market, Data::Ticker(ticker)));
                }
                ResponseData::Order(order) => {
                    self.buf.push_back((response.market, Data::Order(order)));
                }
            }
        }
    }
}

impl Stream for Ws {
    type Item = Result<(Option<Symbol>, Data)>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            if let Some(data) = self.buf.pop_front() {
                return Poll::Ready(Some(Ok(data)));
            }
            let response = {
                // Fetch new response if buffer is empty.
                // safety: this is ok because the future from self.next_response() will only live in this function.
                // It won't be moved anymore.
                let mut next_response = self.next_response();
                let pinned = unsafe { Pin::new_unchecked(&mut next_response) };
                match ready!(pinned.poll(cx)) {
                    Ok(response) => response,
                    Err(e) => {
                        return Poll::Ready(Some(Err(e)));
                    }
                }
            };
            // Handle the response, possibly adding to the buffer
            self.handle_response(response);
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.buf.len(), None)
    }
}
