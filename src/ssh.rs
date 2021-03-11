#[derive(Clone, Debug)]
pub struct SSH {
    host: String,
    options: Vec<String>,
    port: String,
    user: Option<String>,
}

impl SSH {
    pub fn new(
        user_network: bool,
        address: &Option<String>,
        options: &Option<Vec<String>>,
        port: &Option<String>,
        user: &Option<String>,
    ) -> Option<SSH> {
        let host = if let Some(address) = address {
            address.to_string()
        } else if user_network {
            "localhost".to_string()
        } else {
            return None;
        };

        let port = if let Some(port) = port {
            port.to_string()
        } else {
            return None;
        };

        let options = if let Some(options) = options { options.to_owned() } else { Vec::new() };

        let user = user.to_owned();

        Some(SSH { host, options, port, user })
    }

    pub fn user_host(&self, user: &Option<&str>) -> String {
        if let Some(user) = user {
            format!("{}@{}", user, self.host)
        } else if let Some(user) = &self.user {
            format!("{}@{}", user, self.host)
        } else {
            self.host.to_owned()
        }
    }

    pub fn options(&self) -> Vec<&str> {
        let mut options = Vec::with_capacity(self.options.len() * 2);
        for option in &self.options {
            options.push("-o");
            options.push(&option);
        }
        options
    }

    pub fn port(&self) -> &str {
        &self.port
    }
}
