pub struct Consensus {
    rpc: String,
    store: String,
    initial_checkpoint: Vec<u8>,
    pub last_checkpoint: Vec<u8>,
    pub config: String
}