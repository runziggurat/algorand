use std::{io, net::SocketAddr};

use pea2pea::{protocols::Reading, ConnectionSide, Pea2Pea};
use tracing::*;

use crate::{
    protocol::codecs::algomsg::{AlgoMsg, AlgoMsgCodec},
    tools::inner_node::InnerNode,
};

#[async_trait::async_trait]
impl Reading for InnerNode {
    type Message = AlgoMsg;
    type Codec = AlgoMsgCodec;

    fn codec(&self, _addr: SocketAddr, _side: ConnectionSide) -> Self::Codec {
        AlgoMsgCodec::new(self.node().span().clone())
    }

    /// Terminates WebSocket packets, decodes and forwards [AlgoMsg] message to synthetic node's inbound queue.
    async fn process_message(&self, source: SocketAddr, msg: Self::Message) -> io::Result<()> {
        let span = self.node().span();

        debug!(
            parent: span,
            "sending a message received from {source} to the synthetic node's inbound queue: {:?}",
            msg.payload
        );
        self.inbound_tx
            .send((source, msg))
            .await
            .expect("receiver dropped");

        Ok(())
    }
}
