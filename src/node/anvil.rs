// methods to forbid: https://book.getfoundry.sh/reference/anvil/#custom-methods
// Create an anvil listener that will filter forbidden methods and forward the filtered ones to the actual node

use ethers::utils::{Anvil, AnvilInstance};

pub struct HuffPillInstance {
    anvil: AnvilInstance,
    allowed_methods: Vec<String>,
}

impl HuffPillInstance {
    /// Spin up a new HuffPillInstance
    pub fn new(port: u16, allowed_methods: Vec<String>) -> Self {
        Self {
            anvil: start_anvil(port, 30 * 60),
            allowed_methods,
        }
    }

    /// Filter out any custom method that would be cheating
    pub fn filter_out<'a>(method: String, allowed_methods: Vec<String>) -> Option<String> {
        if allowed_methods.contains(&method) {
            Some(method)
        } else {
            None
        }
    }
}

/// Spin up a local anvil node to listen to incoming RPC JSON requests
/// TODO: This instance should be dropped automatically after "timeout" in seconds
pub fn start_anvil(port: u16, timeout: u32) -> AnvilInstance {
    Anvil::new().port(port).spawn()
}
