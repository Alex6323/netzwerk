pub struct Config {
    address: String,
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
}

pub struct ConfigBuilder {
    address: Option<String>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            address: None,
        }
    }
    // TODO: use `impl ToSocketAddrs`
    pub fn with_host_address(mut self, address: String) -> Self {
        self.address.replace(address);
        self
    }

    pub fn build(self) -> Config {
        Config { address: self.address.unwrap_or("localhost:1337".to_string()) }
    }
}