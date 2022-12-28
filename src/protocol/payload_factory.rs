use crate::protocol::codecs::payload::Payload;

/// A factory for creating payloads.
#[derive(Clone)]
pub struct PayloadFactory {
    payload: Payload,
    customize_payload: fn(&mut Payload) -> (),
    pre_generated_cache: Vec<Payload>,
}

impl PayloadFactory {
    /// Create a new payload factory using specified payload as a template and a function to customize
    /// the payload. If no customization is needed, just pass None and use default customizer.
    pub fn new(payload: Payload, customize_payload: Option<fn(&mut Payload) -> ()>) -> Self {
        let default_customize_payload = |msg: &mut Payload| match msg {
            Payload::UniEnsBlockReq(message) => {
                message.nonce += 1;
            }
            Payload::UniCatchupReq(message) => {
                message.nonce += 1;
            }
            _ => {}
        };

        Self {
            payload,
            customize_payload: customize_payload.unwrap_or(default_customize_payload),
            pre_generated_cache: Vec::new(),
        }
    }

    /// Create a new payload with the same type as the template. If there is a need to
    /// change any payload fields customizer is run.
    pub fn generate_next(&mut self) -> Payload {
        (self.customize_payload)(&mut self.payload);
        self.payload.clone()
    }

    /// Create a new payload with the same type as the template and store it in the cache.
    pub fn pre_generate_payloads_cache(&mut self, count: usize) {
        self.pre_generated_cache = self.generate_payloads(count);
    }

    /// Get vector of payloads reference from the cache.
    pub fn get_pre_generated_payload_cache(&self) -> &Vec<Payload> {
        &self.pre_generated_cache
    }

    /// Generate vector of payloads and return it immediately.
    pub fn generate_payloads(&mut self, count: usize) -> Vec<Payload> {
        (0..count).map(|_| self.generate_next()).collect()
    }
}
