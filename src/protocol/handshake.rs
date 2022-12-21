use std::io;

use bytes::Bytes;
use futures_util::{sink::SinkExt, stream::TryStreamExt, StreamExt};
use pea2pea::{protocols::Handshake, Connection, ConnectionSide, Pea2Pea};
use tokio_util::codec::{BytesCodec, Framed};
use tracing::*;

use crate::{protocol::constants::USER_AGENT, tools::inner_node::InnerNode};

pub const X_AG_ALGORAND_VERSION: &str = "2.1";
pub const X_AG_ACCEPT_VERSION: &str = X_AG_ALGORAND_VERSION;
const SEC_WEBSOCKET_VERSION: &str = "13";
const X_AG_INSTANCE_NAME: &str = "synth_node"; // Can be shared between different synthetic nodes
const X_AG_NODE_RANDOM: &str = "cGVhMnBlYQ=="; // Can be shared between different synthetic nodes
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
#[derive(Clone, Debug)]
pub struct SecWebSocket {
    /// Content for the Sec-WebSocket-Key HTTP field.
    pub key: String,
    /// Content for the Sec-WebSocket-Accept HTTP field.
    pub accept: String,
}

impl SecWebSocket {
    /// Generate key-accept pair for a WebSocket handshake.
    pub fn generate() -> Self {
        let key = tungstenite::handshake::client::generate_key();
        let accept = tungstenite::handshake::derive_accept_key(key.as_bytes());
        Self { key, accept }
    }
}

#[derive(Clone, Debug)]
pub struct HandshakeCfg {
    /// WebSocket protocol version.
    pub ws_version: String,
    /// User agent is the HTTP header which identifies the user agent.
    pub user_agent: String,
    /// Node random HTTP header the node uses to make sure it's not talking to itself.
    pub ar_node_random: String,
    /// Genesis HTTP header for genesis ID to identify the chain.
    pub ar_genesis: String,
    /// An HTTP header for protocol version.
    pub ar_version: String,
    /// An HTTP header for the accept protocol version. Client uses this to advertise supported protocol versions.
    pub ar_accept_version: String,
    /// Instance name HTTP header by which an inbound connection reports an ID to distinguish multiple local nodes.
    pub ar_instance_name: String,
    /// Telemetry ID HTTP header for telemetry-id for logging.
    pub ar_tel_id: Option<String>,
    /// Address location HTTP header by which an inbound connection reports its public address.
    pub ar_location: Option<String>,
    /// Network priority challenge sent to clients which try to connect to the node.
    pub challenge: Option<String>,
    /// A key-accept pair for a Sec-WebSocket-Key header.
    pub ws_key: Option<SecWebSocket>,
}

impl Default for HandshakeCfg {
    fn default() -> Self {
        Self {
            ar_instance_name: X_AG_INSTANCE_NAME.into(),
            ws_version: SEC_WEBSOCKET_VERSION.into(),
            user_agent: USER_AGENT.into(),
            ar_node_random: X_AG_NODE_RANDOM.into(),
            ar_genesis: X_AG_ALGORAND_GENESIS.into(),
            ar_accept_version: X_AG_ACCEPT_VERSION.into(),
            ar_version: X_AG_ALGORAND_VERSION.into(),
            // One could use 'd12c01a5-4ca4-4be3-a394-68c8913f3883' as a valid example.
            ar_tel_id: None,
            ar_location: None,
            challenge: None,
            ws_key: None,
        }
    }
}

#[async_trait::async_trait]
impl Handshake for InnerNode {
    async fn perform_handshake(&self, mut conn: Connection) -> io::Result<Connection> {
        let conn_addr = conn.addr();
        let node_conn_side = !conn.side();
        let stream = self.borrow_stream(&mut conn);
        let cfg = &self.handshake_cfg;

        match node_conn_side {
            ConnectionSide::Initiator => {
                let mut framed = Framed::new(stream, BytesCodec::default());

                let sec_ws = if let Some(ws_key) = self.handshake_cfg.ws_key.clone() {
                    ws_key
                } else {
                    SecWebSocket::generate()
                };

                let mut req = Vec::new();
                let mut req_header = |mut header: String| {
                    header.push_str("\r\n");
                    req.extend_from_slice(header.as_bytes());
                };

                req_header(format!("GET /v1/{}/gossip HTTP/1.1", X_AG_ALGORAND_GENESIS));
                req_header(format!("Host: {}", conn_addr));
                req_header(format!("User-Agent: {}", cfg.user_agent));
                req_header("Connection: Upgrade".into());
                req_header(format!("Sec-WebSocket-Key: {}", sec_ws.key));
                req_header(format!("Sec-WebSocket-Version: {}", cfg.ws_version));
                req_header("Upgrade: websocket".into());
                req_header(format!(
                    "X-Algorand-Accept-Version: {}",
                    cfg.ar_accept_version
                ));
                req_header(format!("X-Algorand-Instancename: {}", cfg.ar_instance_name));
                if let Some(ref location) = cfg.ar_location {
                    req_header(format!("X-Algorand-Location: {location}"));
                }
                req_header(format!("X-Algorand-Noderandom: {}", cfg.ar_node_random));
                if let Some(ref telid) = cfg.ar_tel_id {
                    req_header(format!("X-Algorand-Telid: {telid}"));
                }
                req_header(format!("X-Algorand-Version: {}", cfg.ar_version));
                req_header(format!("X-Algorand-Genesis: {}", cfg.ar_genesis));
                req_header("".into()); // A HTTP header ends with '\r\n'

                let req = Bytes::from(req);
                info!(parent: self.node().span(), "sending a handshake request: {:?}", req);
                framed.send(req).await.unwrap();

                let rsp = framed.try_next().await.unwrap().unwrap();
                info!(parent: self.node().span(), "received a handshake response: {:?}", rsp);

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
                        error!(parent: self.node().span(), "invalid Sec-WebSocket-Accept");
                        return Err(io::ErrorKind::InvalidData.into());
                    }
                    trace!(parent: self.node().span(), "valid Sec-WebSocket-Accept");
                } else {
                    error!(parent: self.node().span(), "missing Sec-WebSocket-Accept");
                    return Err(io::ErrorKind::InvalidData.into());
                };
            }
            ConnectionSide::Responder => {
                let peer_addr = stream.peer_addr().unwrap();
                let mut framed = Framed::new(stream, BytesCodec::default());

                let req = framed.next().await.unwrap().unwrap();
                info!(parent: self.node().span(), "{:?}: received a handshake request: {:?}", peer_addr, req);

                let mut req_headers = [httparse::EMPTY_HEADER; 32];
                let mut parsed_req = httparse::Request::new(&mut req_headers);
                parsed_req.parse(&req).unwrap();

                let swa = if let Some(ws_key) = self.handshake_cfg.ws_key.clone() {
                    ws_key.accept
                } else if let Some(swk) = parsed_req
                    .headers
                    .iter()
                    .find(|h| h.name.to_ascii_lowercase() == "sec-websocket-key")
                {
                    tungstenite::handshake::derive_accept_key(swk.value)
                } else {
                    error!(parent: self.node().span(), "missing Sec-WebSocket-Key");
                    return Err(io::ErrorKind::InvalidData.into());
                };

                let mut rsp = Vec::new();
                let mut rsp_header = |mut header: String| {
                    header.push_str("\r\n");
                    rsp.extend_from_slice(header.as_bytes());
                };

                rsp_header("HTTP/1.1 101 Switching Protocols".into());
                rsp_header("Upgrade: websocket".into());
                rsp_header("Connection: Upgrade".into());
                rsp_header(format!("Sec-Websocket-Accept: {swa}"));
                rsp_header(format!("X-Algorand-Instancename: {}", cfg.ar_instance_name));
                if let Some(ref location) = cfg.ar_location {
                    rsp_header(format!("X-Algorand-Location: {location}"));
                }
                rsp_header(format!("X-Algorand-Noderandom: {}", cfg.ar_node_random));
                rsp_header(format!("X-Algorand-Version: {}", cfg.ar_accept_version));
                rsp_header(format!("X-Algorand-Genesis: {}", cfg.ar_genesis));
                if let Some(ref challenge) = cfg.challenge {
                    rsp_header(format!("X-Algorand-Prioritychallenge: {challenge}"));
                }
                rsp_header("".into()); // A HTTP header ends with '\r\n'

                let rsp = Bytes::from(rsp);
                info!(parent: self.node().span(), "sending a handshake response: {:?}", rsp);
                framed.send(rsp).await.unwrap();
            }
        }

        Ok(conn)
    }
}
