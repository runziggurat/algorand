use std::{io, net::SocketAddr};

use pea2pea::{protocols::Reading, ConnectionSide, Pea2Pea};
use tracing::*;

use crate::{protocol::codecs::websocketcodec::WebsocketCodec, tools::inner_node::InnerNode};

#[async_trait::async_trait]
impl Reading for InnerNode {
    type Message = websocket_codec::Message;
    type Codec = WebsocketCodec;

    fn codec(&self, _addr: SocketAddr, _side: ConnectionSide) -> Self::Codec {
        Default::default()
    }

    async fn process_message(&self, source: SocketAddr, message: Self::Message) -> io::Result<()> {
        info!(parent: self.node().span(), "got a message from {}: {:?}", source, message);
        debug!(
        parent: self.node().span(),
        "sending the message to the node's inbound queue"
        );
        self.inbound_tx
            .send((source, message))
            .await
            .expect("receiver dropped");
        Ok(())
    }
}
