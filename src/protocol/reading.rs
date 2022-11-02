use std::{io, net::SocketAddr};

use pea2pea::{protocols::Reading, ConnectionSide, Pea2Pea};
use tracing::*;

use crate::{
    protocol::codecs::{algomsg::AlgoMsgCodec, payload::Payload},
    tools::inner_node::InnerNode,
};

#[async_trait::async_trait]
impl Reading for InnerNode {
    type Message = Payload;
    type Codec = AlgoMsgCodec;

    fn codec(&self, _addr: SocketAddr, _side: ConnectionSide) -> Self::Codec {
        AlgoMsgCodec::new(self.node().span().clone())
    }

    /// Terminates WebSocket packets, decodes and forwards algod message [Payload] to synthetic node's inbound queue.
    async fn process_message(&self, source: SocketAddr, payload: Self::Message) -> io::Result<()> {
        let span = self.node().span();

        debug!(parent: span, "got a message from {}", source);
        debug!(
            parent: span,
            "sending the message to the node's inbound queue: {:?}", payload
        );
        self.inbound_tx
            .send((source, payload))
            .await
            .expect("receiver dropped");

        Ok(())
    }
}
