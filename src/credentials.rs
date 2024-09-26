use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Read, Write};
use toml;
use crate::servers::RithmicServer;
use crate::systems::RithmicSystem;

#[derive(Debug, Deserialize, Serialize)]
pub struct RithmicCredentials {
    pub user: String,
    pub server_name: RithmicServer,
    pub system_name: RithmicSystem,
    pub password: String,
}

impl RithmicCredentials {
    pub fn save_credentials_to_file(&self, file_path: &str) -> io::Result<()> {
        // Convert the credentials to TOML string
        let toml_string = toml::to_string(self).expect("Failed to serialize credentials");

        // Write the TOML string to the file
        let mut file = File::create(file_path)?;
        file.write_all(toml_string.as_bytes())?;

        Ok(())
    }

    pub fn load_credentials_from_file(file_path: &str) -> io::Result<RithmicCredentials> {
        // Read the TOML string from the file
        let mut file = File::open(file_path)?;
        let mut toml_string = String::new();
        file.read_to_string(&mut toml_string)?;

        // Parse the TOML string into Credentials
        let credentials: RithmicCredentials = toml::de::from_str(&toml_string)
            .expect("Failed to deserialize credentials");

        Ok(credentials)
    }

    pub fn file_name(&self) -> String {
        self.system_name.file_string()
    }
}
