use std::io;

use bytes::Bytes;
use futures_util::{sink::SinkExt, stream::TryStreamExt, StreamExt};
use pea2pea::{protocols::Handshake, Connection, ConnectionSide, Pea2Pea};
use tokio_util::codec::{BytesCodec, Framed};
use tracing::*;

use crate::{protocol::constants::USER_AGENT, tools::inner_node::InnerNode};

const SEC_WEBSOCKET_VERSION: &str = "13";
const X_AG_ACCEPT_VERSION: &str = "2.1";
const X_AG_INSTANCE_NAME: &str = "synth_node"; // Can be shared between different synthetic nodes
const X_AG_NODE_RANDOM: &str = "cGVhMnBlYQ=="; // Can be shared between different synthetic nodes
const X_AG_ALGORAND_VERSION: &str = "2.1";
const X_AG_ALGORAND_GENESIS: &str = "private-v1";

// Info from RFC 6455, section 4.1, page 18:
//
// The request MUST include a header field with the name
// |Sec-WebSocket-Key|.  The value of this header field MUST be a
// nonce consisting of a randomly selected 16-byte value that has
// been base64-encoded (see Section 4 of [RFC4648]).  The nonce
// MUST be selected randomly for each connection.
//
// NOTE: As an example, if the randomly selected value was the
// sequence of bytes 0x01 0x02 0x03 0x04 0x05 0x06 0x07 0x08 0x09
// 0x0a 0x0b 0x0c 0x0d 0x0e 0x0f 0x10, the value of the header
// field would be "AQIDBAUGBwgJCgsMDQ4PEC=="
/// A key-accept pair for a Sec-WebSocket-Key header.
struct SecWebSocket {
    key: String,
    accept: String,
}

impl SecWebSocket {
    /// Generate key-accept pair for a WebSocket handshake.
    fn generate() -> Self {
        let key = tungstenite::handshake::client::generate_key();
        let accept = tungstenite::handshake::derive_accept_key(key.as_bytes());
        Self { key, accept }
    }
}

#[async_trait::async_trait]
impl Handshake for InnerNode {
    async fn perform_handshake(&self, mut conn: Connection) -> io::Result<Connection> {
        let conn_addr = conn.addr();
        let node_conn_side = !conn.side();
        let stream = self.borrow_stream(&mut conn);

        match node_conn_side {
            ConnectionSide::Initiator => {
                let mut framed = Framed::new(stream, BytesCodec::default());
                let sec_ws = SecWebSocket::generate();

                let mut req = Vec::new();
                req.extend_from_slice(b"GET /v1/private-v1/gossip HTTP/1.1\r\n");
                req.extend_from_slice(format!("Host: {}\r\n", conn_addr).as_bytes());
                req.extend_from_slice(format!("User-Agent: {}\r\n", USER_AGENT).as_bytes());
                req.extend_from_slice(b"Connection: Upgrade\r\n");
                req.extend_from_slice(format!("Sec-WebSocket-Key: {}\r\n", sec_ws.key).as_bytes());
                req.extend_from_slice(
                    format!("Sec-WebSocket-Version: {}\r\n", SEC_WEBSOCKET_VERSION).as_bytes(),
                );
                req.extend_from_slice(b"Upgrade: websocket\r\n");
                req.extend_from_slice(
                    format!("X-Algorand-Accept-Version: {}\r\n", X_AG_ACCEPT_VERSION).as_bytes(),
                );
                req.extend_from_slice(
                    format!("X-Algorand-Instancename: {}\r\n", X_AG_INSTANCE_NAME).as_bytes(),
                );
                req.extend_from_slice(b"X-Algorand-Location: \r\n");
                req.extend_from_slice(
                    format!("X-Algorand-Noderandom: {}\r\n", X_AG_NODE_RANDOM).as_bytes(),
                );
                // req.extend_from_slice(b"X-Algorand-Telid: d12c01a5-4ca4-4be3-a394-68c8913f3883\r\n"); // TODO: Investigate more
                req.extend_from_slice(
                    format!("X-Algorand-Version: {}\r\n", X_AG_ALGORAND_VERSION).as_bytes(),
                );
                req.extend_from_slice(
                    format!("X-Algorand-Genesis: {}\r\n", X_AG_ALGORAND_GENESIS).as_bytes(),
                );
                req.extend_from_slice(b"\r\n");
                let req = Bytes::from(req);

                info!(parent: self.node().span(), "sending handshake message: {:?}", req);
                framed.send(req).await.unwrap();

                let rsp = framed.try_next().await.unwrap().unwrap();
                info!(parent: self.node().span(), "received handshake message: {:?}", rsp);

                let mut rsp_headers = [httparse::EMPTY_HEADER; 32];
                let mut parsed_rsp = httparse::Response::new(&mut rsp_headers);
                parsed_rsp.parse(&rsp).unwrap();

                // Verify Sec-Websocket-Accept
                if let Some(swk) = parsed_rsp
                    .headers
                    .iter()
                    .find(|h| h.name.to_ascii_lowercase() == "sec-websocket-accept")
                {
                    if sec_ws.accept.as_bytes() != swk.value {
                        error!(parent: self.node().span(), "invalid Sec-WebSocket-Accept!");
                        return Err(io::ErrorKind::InvalidData.into());
                    }
                    trace!(parent: self.node().span(), "valid Sec-WebSocket-Accept");
                } else {
                    error!(parent: self.node().span(), "missing Sec-WebSocket-Accept!");
                    return Err(io::ErrorKind::InvalidData.into());
                };
            }
            ConnectionSide::Responder => {
                let peer_addr = stream.peer_addr().unwrap();
                let mut framed = Framed::new(stream, BytesCodec::default());

                let req = framed.next().await.unwrap().unwrap();
                info!(parent: self.node().span(), "{:?}: received handshake message: {:?}", peer_addr, req);

                let mut req_headers = [httparse::EMPTY_HEADER; 32];
                let mut parsed_req = httparse::Request::new(&mut req_headers);
                parsed_req.parse(&req).unwrap();

                let swa = if let Some(swk) = parsed_req
                    .headers
                    .iter()
                    .find(|h| h.name.to_ascii_lowercase() == "sec-websocket-key")
                {
                    tungstenite::handshake::derive_accept_key(swk.value)
                } else {
                    error!(parent: self.node().span(), "missing Sec-WebSocket-Key!");
                    return Err(io::ErrorKind::InvalidData.into());
                };

                let mut rsp = Vec::new();
                rsp.extend_from_slice(b"HTTP/1.1 101 Switching Protocols\r\n");
                rsp.extend_from_slice(b"Upgrade: websocket\r\n");
                rsp.extend_from_slice(b"Connection: Upgrade\r\n");
                rsp.extend_from_slice(format!("Sec-Websocket-Accept: {}\r\n", swa).as_bytes());
                rsp.extend_from_slice(
                    format!("X-Algorand-Instancename: {}\r\n", X_AG_INSTANCE_NAME).as_bytes(),
                );
                rsp.extend_from_slice(b"X-Algorand-Location:\r\n");
                rsp.extend_from_slice(
                    format!("X-Algorand-Noderandom: {}\r\n", X_AG_NODE_RANDOM).as_bytes(),
                );
                rsp.extend_from_slice(
                    format!("X-Algorand-Version: {}\r\n", X_AG_ACCEPT_VERSION).as_bytes(),
                );
                rsp.extend_from_slice(
                    format!("X-Algorand-Genesis: {}\r\n", X_AG_ALGORAND_GENESIS).as_bytes(),
                );
                rsp.extend_from_slice(b"\r\n");
                let rsp = Bytes::from(rsp);

                info!(parent: self.node().span(), "sending handshake message: {:?}", rsp);
                framed.send(rsp).await.unwrap();
            }
        }

        Ok(conn)
    }
}
