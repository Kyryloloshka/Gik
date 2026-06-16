use crate::error::{Result, GikError};
use crate::core::hash::Hash;
use ureq::{Agent, AgentBuilder};
use std::time::Duration;
use base64::{Engine as _, engine::general_purpose::STANDARD};

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

    pub fn discover_refs(&self, branch: &str) -> Result<Option<Hash>> {
        let req_url = format!("{}/info/refs?service=git-receive-pack", self.url);
        let mut req = self.agent.get(&req_url);
        
        if let Some(t) = &self.token {
            let auth = STANDARD.encode(format!("git:{}", t));
            req = req.set("Authorization", &format!("Basic {}", auth));
        }

        let resp = req.call().map_err(|e| GikError::Io(std::io::Error::other(e.to_string())))?;
        
        if resp.status() != 200 {
            return Err(GikError::Io(std::io::Error::other(format!("HTTP Error: {}", resp.status()))));
        }

        let body = resp.into_string().map_err(|e| GikError::Io(std::io::Error::other(e)))?;
        
        Ok(parse_discover_refs_body(&body, branch))
    }

    pub fn push_packfile<R: std::io::Read>(&self, local_head: &Hash, remote_head: Option<&Hash>, mut packfile: R, branch: &str) -> Result<()> {
        let req_url = format!("{}/git-receive-pack", self.url);
        let mut req = self.agent.post(&req_url)
            .set("Content-Type", "application/x-git-receive-pack-request");
            
        if let Some(t) = &self.token {
            use base64::{Engine as _, engine::general_purpose::STANDARD};
            let auth = STANDARD.encode(format!("git:{}", t));
            req = req.set("Authorization", &format!("Basic {}", auth));
        }

        // Pkt-line format: <old_hash> <new_hash> refs/heads/<branch>\0report-status
        let old_hash = remote_head.map(|h| h.to_string()).unwrap_or_else(|| "0000000000000000000000000000000000000000".to_string());
        let new_hash = local_head.to_string();
        let cmd = format!("{} {} refs/heads/{}\0report-status", old_hash, new_hash, branch);
        
        // pkt-line prefix is length in hex (cmd length + 4)
        let pkt_len = cmd.len() + 4;
        let pkt_line = format!("{:04x}{}", pkt_len, cmd);
        
        // Then standard flush packet "0000"
        let flush_pkt = "0000";
        
        let mut prefix = Vec::new();
        prefix.extend_from_slice(pkt_line.as_bytes());
        prefix.extend_from_slice(flush_pkt.as_bytes());

        use std::io::Read;
        let reader = std::io::Cursor::new(prefix).chain(packfile);

        let resp = req.send(reader).map_err(|e| GikError::Io(std::io::Error::other(e.to_string())))?;
        
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
        let client = GitClient::new("https://github.com/test".to_string(), None);
        assert_eq!(client.url, "https://github.com/test");
        assert_eq!(client.token, None);
    }

    #[test]
    fn test_parse_discover_refs_body_found() {
        let body = "001e# service=git-receive-pack\n\
                    000000a4fd4bf91c3700325211bbcf1b35f2178f04552506 HEAD\\0multi_ack ...\n\
                    003ffd4bf91c3700325211bbcf1b35f2178f04552506 refs/heads/main\n\
                    003f1b2871621c5b764b8637fc5873a17989e7f0805d refs/heads/soska\n\
                    0000";
        
        let hash_main = parse_discover_refs_body(body, "main").unwrap();
        assert_eq!(hash_main.to_string(), "fd4bf91c3700325211bbcf1b35f2178f04552506");

        let hash_soska = parse_discover_refs_body(body, "soska").unwrap();
        assert_eq!(hash_soska.to_string(), "1b2871621c5b764b8637fc5873a17989e7f0805d");
    }

    #[test]
    fn test_parse_discover_refs_body_not_found() {
        let body = "001e# service=git-receive-pack\n0000";
        assert_eq!(parse_discover_refs_body(body, "main"), None);
    }
}

fn parse_discover_refs_body(body: &str, branch: &str) -> Option<Hash> {
    let search_target = format!("refs/heads/{}", branch);
    for line in body.lines() {
        if line.contains(&search_target) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(hash_str) = parts.first() {
                let clean_hash = if hash_str.len() > 40 { &hash_str[hash_str.len()-40..] } else { hash_str };
                if let Ok(hash) = Hash::from_hex(clean_hash) {
                    return Some(hash);
                }
            }
        }
    }
    None
}
