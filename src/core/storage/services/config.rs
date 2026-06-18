use crate::core::storage::repository::*;
use crate::error::Result;
use redb::ReadableTable;
use std::collections::HashMap;

pub struct ConfigService<'a> {
    pub(crate) repo: &'a Repository,
}

impl<'a> ConfigService<'a> {
    pub fn get_local(&self, key: &str) -> Result<Option<String>> {
        let read_txn = self.repo.db.begin_read()?;
        let table = read_txn.open_table(CONFIG)?;
        let val = table.get(key)?.map(|g| g.value().to_string());
        Ok(val)
    }

    pub fn set_local(&self, key: &str, value: &str) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            let mut table = write_txn.open_table(CONFIG)?;
            table.insert(key, value)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_global(&self, key: &str) -> Result<Option<String>> {
        let config_path = dirs::home_dir()
            .ok_or_else(|| crate::error::GikError::Config("Home directory not found".to_string()))?
            .join(".gikconfig");

        if !config_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(config_path)?;
        for line in content.lines() {
            if let Some((k, v)) = line.split_once('=') {
                if k.trim() == key {
                    return Ok(Some(v.trim().to_string()));
                }
            }
        }
        Ok(None)
    }

    pub fn set_global(&self, key: &str, value: &str) -> Result<()> {
        let config_path = dirs::home_dir()
            .ok_or_else(|| crate::error::GikError::Config("Home directory not found".to_string()))?
            .join(".gikconfig");

        let mut config_map = HashMap::new();
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            for line in content.lines() {
                if let Some((k, v)) = line.split_once('=') {
                    config_map.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
        }

        config_map.insert(key.to_string(), value.to_string());

        let mut new_content = String::new();
        for (k, v) in config_map {
            new_content.push_str(&format!("{}={}\n", k, v));
        }

        std::fs::write(config_path, new_content)?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<String>> {
        if let Some(val) = self.get_local(key)? {
            return Ok(Some(val));
        }
        self.get_global(key)
    }
}
