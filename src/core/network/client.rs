use crate::core::hash::Hash;
use crate::error::{GikError, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::time::Duration;
use ureq::{Agent, AgentBuilder};

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

    fn apply_auth(&self, mut req: ureq::Request) -> ureq::Request {
        if let Some(t) = &self.token {
            let auth = STANDARD.encode(format!("git:{}", t));
            req = req.set("Authorization", &format!("Basic {}", auth));
        }
        req
    }

    pub fn discover_refs(&self, branch: &str) -> Result<Option<Hash>> {
        let req_url = format!("{}/info/refs?service=git-receive-pack", self.url);
        let req = self.apply_auth(self.agent.get(&req_url));

        let resp = req.call().map_err(|e| GikError::Network(e.to_string()))?;

        if resp.status() == 401 {
            return Err(GikError::Auth(
                "Invalid token or authentication required".to_string(),
            ));
        } else if resp.status() != 200 {
            return Err(GikError::Network(format!("HTTP Error: {}", resp.status())));
        }

        let body = resp
            .into_string()
            .map_err(|e| GikError::Network(e.to_string()))?;

        Ok(parse_discover_refs_body(&body, branch))
    }

    pub fn push_packfile<R: std::io::Read>(
        &self,
        local_head: &Hash,
        remote_head: Option<&Hash>,
        packfile: R,
        branch: &str,
    ) -> Result<()> {
        let req_url = format!("{}/git-receive-pack", self.url);
        let req = self
            .agent
            .post(&req_url)
            .set("Content-Type", "application/x-git-receive-pack-request");

        let req = self.apply_auth(req);

        // Pkt-line format: <old_hash> <new_hash> refs/heads/<branch>\0report-status
        let old_hash = remote_head
            .map(|h| h.to_string())
            .unwrap_or_else(|| "0000000000000000000000000000000000000000".to_string());
        let new_hash = local_head.to_string();
        let cmd = format!(
            "{} {} refs/heads/{}\0report-status",
            old_hash, new_hash, branch
        );

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

        let resp = req
            .send(reader)
            .map_err(|e| GikError::Network(e.to_string()))?;

        if resp.status() == 401 {
            return Err(GikError::Auth(
                "Invalid token or authentication required during push".to_string(),
            ));
        } else if resp.status() != 200 {
            return Err(GikError::Network(format!(
                "HTTP Error during push: {}",
                resp.status()
            )));
        }

        // Print remote response (e.g., unpack ok)
        let body = resp
            .into_string()
            .map_err(|e| GikError::Network(e.to_string()))?;
        println!("Server responded:\n{}", body);

        Ok(())
    }

    pub fn discover_fetch_refs(&self, branch: &str) -> Result<Option<Hash>> {
        let req_url = format!("{}/info/refs?service=git-upload-pack", self.url);
        let req = self.apply_auth(self.agent.get(&req_url));

        let resp = req
            .call()
            .map_err(|e| crate::error::GikError::Network(e.to_string()))?;
        if resp.status() == 401 {
            return Err(crate::error::GikError::Auth(
                "Invalid token or authentication required".to_string(),
            ));
        } else if resp.status() != 200 {
            return Err(crate::error::GikError::Network(format!(
                "HTTP Error: {}",
                resp.status()
            )));
        }

        let body = resp
            .into_string()
            .map_err(|e| crate::error::GikError::Network(e.to_string()))?;
        Ok(parse_discover_refs_body(&body, branch))
    }

    pub fn fetch_packfile(
        &self,
        want_hash: &Hash,
        have_hash: Option<&Hash>,
    ) -> Result<Box<dyn std::io::Read + Send>> {
        let req_url = format!("{}/git-upload-pack", self.url);
        let req = self
            .agent
            .post(&req_url)
            .set("Content-Type", "application/x-git-upload-pack-request");

        let req = self.apply_auth(req);

        let want_cmd = format!("want {} multi_ack\n", want_hash);
        let pkt_len = want_cmd.len() + 4;
        let mut body_str = format!("{:04x}{}", pkt_len, want_cmd);

        body_str.push_str("0000"); // Flush packet separates want and have

        if let Some(h) = have_hash {
            let have_cmd = format!("have {}\n", h);
            let h_len = have_cmd.len() + 4;
            body_str.push_str(&format!("{:04x}{}", h_len, have_cmd));
        }

        body_str.push_str("0009done\n");

        let resp = req
            .send_string(&body_str)
            .map_err(|e| crate::error::GikError::Network(e.to_string()))?;

        if resp.status() == 401 {
            return Err(crate::error::GikError::Auth(
                "Invalid token or authentication required".to_string(),
            ));
        } else if resp.status() != 200 {
            return Err(crate::error::GikError::Network(format!(
                "HTTP Error: {}",
                resp.status()
            )));
        }

        use std::io::BufRead;
        let mut reader = std::io::BufReader::new(resp.into_reader());

        // Read lines until we hit "PACK"
        loop {
            let buffer = reader.fill_buf()?;
            if buffer.len() < 4 {
                break;
            }
            if &buffer[0..4] == b"PACK" {
                break;
            }

            // It's a pkt-line. Parse its length.
            let len_str = std::str::from_utf8(&buffer[0..4]).unwrap_or("0000");
            let pkt_len = usize::from_str_radix(len_str, 16).unwrap_or(0);

            if pkt_len == 0 {
                reader.consume(4); // flush pkt
                continue;
            }

            // Read the full pkt-line
            let mut pkt_buf = vec![0u8; pkt_len];
            std::io::Read::read_exact(&mut reader, &mut pkt_buf)?;

            let line = String::from_utf8_lossy(&pkt_buf);
            if line.contains("NAK") && have_hash.is_some() {
                println!("Remote did not recognize our local HEAD. Falling back to full fetch.");
            }
            println!("Server sent: {}", line.trim_end());
        }

        Ok(Box::new(reader))
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
        assert_eq!(
            hash_main.to_string(),
            "fd4bf91c3700325211bbcf1b35f2178f04552506"
        );

        let hash_soska = parse_discover_refs_body(body, "soska").unwrap();
        assert_eq!(
            hash_soska.to_string(),
            "1b2871621c5b764b8637fc5873a17989e7f0805d"
        );
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
                let clean_hash = if hash_str.len() > 40 {
                    &hash_str[hash_str.len() - 40..]
                } else {
                    hash_str
                };
                if let Ok(hash) = Hash::from_hex(clean_hash) {
                    return Some(hash);
                }
            }
        }
    }
    None
}
