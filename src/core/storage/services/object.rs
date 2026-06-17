use crate::error::Result;
use crate::core::hash::Hash;
use crate::core::storage::repository::*;
use redb::ReadableTable;

pub struct ObjectService<'a> {
    pub(crate) repo: &'a Repository,
}

impl<'a> ObjectService<'a> {
    pub fn contains_object(&self, hash: &Hash) -> Result<bool> {
        let read_txn = self.repo.db.begin_read()?;
        let table = read_txn.open_table(OBJECTS)?;
        let exists = table.get(&hash.0)?.is_some();
        Ok(exists)
    }

    pub fn write_object(&self, hash: &Hash, compressed_data: &[u8]) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            let mut table = write_txn.open_table(OBJECTS)?;
            table.insert(&hash.0, compressed_data.to_vec())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn list_all_objects(&self) -> Result<Vec<Hash>> {
        let read_txn = self.repo.db.begin_read()?;
        let table = read_txn.open_table(OBJECTS)?;
        let mut hashes = Vec::new();
        for result in table.iter()? {
            let (hash_bytes, _) = result?;
            hashes.push(Hash(*hash_bytes.value()));
        }
        Ok(hashes)
    }

    pub fn get_object(&self, hash: &Hash) -> Result<Option<Vec<u8>>> {
        let read_txn = self.repo.db.begin_read()?;
        let table = read_txn.open_table(OBJECTS)?;
        let guard = table.get(&hash.0)?;
        Ok(guard.map(|g| g.value()))
    }

    pub fn get_blob_text(&self, hash: &Hash) -> Result<String> {
        if let Some(compressed_data) = self.get_object(hash)? {
            let (obj_type, _size, content) = crate::core::objects::decompress_object(&compressed_data[..])?;
            if obj_type != "blob" {
                return Err(crate::error::GikError::Validation(format!("Object {} is not a blob (type: {})", hash, obj_type)));
            }
            Ok(String::from_utf8_lossy(&content).to_string())
        } else {
            return Err(crate::error::GikError::NotFound(format!("Loose object {} not found", hash)));
        }
    }
}
