use crate::protocol::codecs::payload::Payload;

/// A factory for creating payloads.
#[derive(Clone)]
pub struct PayloadFactory {
    payload: Payload,
}

impl PayloadFactory {
    /// Create a new payload factory using specified payload as a template.
    pub fn new(payload: Payload) -> Self {
        Self { payload }
    }

    // TODO[asmie]: Add generate() method to start generating payloads (first message).
    // TODO[asmie]: Re-think API of the PayloadFactory to handle all generic cases (like creating custom payloads with custom fields).

    /// Create a new payload with the same type as the template. If there is a need to
    /// change any payload fields it's done here.
    pub fn generate_next(&mut self) -> Payload {
        // For now, we're incrementing nonce in UniEnsBlockReq and not change other
        // type of messages.
        if let Payload::UniEnsBlockReq(message) = &mut self.payload {
            message.nonce += 1;
        }

        self.payload.clone()
    }
}
