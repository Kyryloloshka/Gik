use crate::error::{Result, GikError};
use crate::core::hash::Hash;
use ureq::{Agent, AgentBuilder};
use std::time::Duration;

pub struct GitClient {
    url: String,
    token: Option<String>,
    agent: Agent,
}

impl GitClient {
    pub fn new(url: String, token: Option<String>) -> Self {
        let agent = AgentBuilder::new()
            .timeout_read(Duration::from_secs(60))
            .timeout_write(Duration::from_secs(60))
            .build();
        Self { url, token, agent }
    }

    pub fn discover_refs(&self) -> Result<Option<Hash>> {
        let req_url = format!("{}/info/refs?service=git-receive-pack", self.url);
        let mut req = self.agent.get(&req_url);
        
        if let Some(t) = &self.token {
            req = req.set("Authorization", &format!("Bearer {}", t));
        }

        let resp = req.call().map_err(|e| GikError::Io(std::io::Error::other(e.to_string())))?;
        
        if resp.status() != 200 {
            return Err(GikError::Io(std::io::Error::other(format!("HTTP Error: {}", resp.status()))));
        }

        let body = resp.into_string().map_err(|e| GikError::Io(std::io::Error::other(e)))?;
        
        // Very basic pkt-line parsing for refs/heads/main
        for line in body.lines() {
            if line.contains("refs/heads/main") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(hash_str) = parts.first() {
                    // Extract exactly 40 chars of hash (Git pkt-line prefixes with length, e.g., "003e<hash>")
                    let clean_hash = if hash_str.len() > 40 { &hash_str[hash_str.len()-40..] } else { hash_str };
                    if let Ok(hash) = Hash::from_hex(clean_hash) {
                        return Ok(Some(hash));
                    }
                }
            }
        }
        
        Ok(None)
    }

    pub fn push_packfile(&self, local_head: &Hash, remote_head: Option<&Hash>, packfile: &[u8]) -> Result<()> {
        let req_url = format!("{}/git-receive-pack", self.url);
        let mut req = self.agent.post(&req_url)
            .set("Content-Type", "application/x-git-receive-pack-request");
            
        if let Some(t) = &self.token {
            req = req.set("Authorization", &format!("Bearer {}", t));
        }

        // Pkt-line format: <old_hash> <new_hash> refs/heads/main\0report-status
        let old_hash = remote_head.map(|h| h.to_string()).unwrap_or_else(|| "0000000000000000000000000000000000000000".to_string());
        let new_hash = local_head.to_string();
        let cmd = format!("{} {} refs/heads/main\0", old_hash, new_hash);
        
        // pkt-line prefix is length in hex (cmd length + 4)
        let pkt_len = cmd.len() + 4;
        let pkt_line = format!("{:04x}{}", pkt_len, cmd);
        
        // Then standard flush packet "0000"
        let flush_pkt = "0000";
        
        let mut payload = Vec::new();
        payload.extend_from_slice(pkt_line.as_bytes());
        payload.extend_from_slice(flush_pkt.as_bytes());
        payload.extend_from_slice(packfile);

        let resp = req.send_bytes(&payload).map_err(|e| GikError::Io(std::io::Error::other(e.to_string())))?;
        
        if resp.status() != 200 {
            return Err(GikError::Io(std::io::Error::other(format!("HTTP Error during push: {}", resp.status()))));
        }
        
        // Print remote response (e.g., unpack ok)
        let body = resp.into_string().map_err(|e| GikError::Io(std::io::Error::other(e)))?;
        println!("Server responded:\n{}", body);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_init() {
        let client = GitClient::new("https://example.com/repo.git".to_string(), None);
        assert_eq!(client.url, "https://example.com/repo.git");
    }
}
