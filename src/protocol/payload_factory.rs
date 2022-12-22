use crate::protocol::codecs::payload::Payload;

/// A factory for creating payloads.
#[derive(Clone)]
pub struct PayloadFactory {
    payload: Payload,
}

impl PayloadFactory {
    /// Create a new payload factory using specified payload as a template.
    pub fn new(payload_type: Payload) -> Self {
        Self {
            payload: payload_type,
        }
    }

    // TODO[asmie]: Add generate() method to start generating payloads (first message).
    // TODO[asmie]: Re-think API of the PayloadFactory to handle all generic cases (like creating custom payloads with custom fields).

    /// Create a new payload with the same type as the template. If there is a need to
    /// change any payload fields it's done here.
    pub fn generate_next(&self) -> Payload {
        let msg = self.payload.clone();

        // For now, we're incrementing nonce in UniEnsBlockReq and not change other
        // type of messages.
        match msg {
            Payload::UniEnsBlockReq(mut message) => {
                message.nonce += 1;
                Payload::UniEnsBlockReq(message)
            }
            _ => msg,
        }
    }
}
