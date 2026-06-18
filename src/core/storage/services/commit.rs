use crate::error::Result;
use crate::core::hash::Hash;
use crate::core::storage::repository::*;
use redb::ReadableTable;

pub struct CommitService<'a> {
    pub(crate) repo: &'a Repository,
}

impl<'a> CommitService<'a> {
    pub fn get_current_head(&self) -> Result<Option<Hash>> {
        let read_txn = self.repo.db.begin_read()?;
        let table = read_txn.open_table(HEADS)?;
        let mut heads = Vec::new();
        for result in table.iter()? {
            let (hash, _) = result?;
            heads.push(Hash(*hash.value()));
        }
        Ok(heads.first().copied())
    }

    pub fn get_commit_meta(&self, hash: &Hash) -> Result<Option<crate::core::models::CommitMeta>> {
        let read_txn = self.repo.db.begin_read()?;
        let table = read_txn.open_table(COMMITS_METADATA)?;
        let guard = table.get(&hash.0)?;
        if let Some(g) = guard {
            let meta = bincode::deserialize(&g.value())?;
            Ok(Some(meta))
        } else {
            Ok(None)
        }
    }

    pub fn commit_transaction(
        &self,
        commit_hash: Hash,
        parent_hash: Option<Hash>,
        meta: crate::core::models::CommitMeta,
    ) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            let mut metadata = write_txn.open_table(COMMITS_METADATA)?;
            let meta_bytes = bincode::serialize(&meta)?;
            metadata.insert(&commit_hash.0, meta_bytes)?;

            let mut heads = write_txn.open_table(HEADS)?;
            if let Some(parent) = parent_hash {
                heads.remove(&parent.0)?;
            }
            heads.insert(&commit_hash.0, 1)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn insert_commit_meta(&self, hash: &Hash, meta: crate::core::models::CommitMeta) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            let mut metadata = write_txn.open_table(COMMITS_METADATA)?;
            let meta_bytes = bincode::serialize(&meta)?;
            metadata.insert(&hash.0, meta_bytes)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn set_head(&self, new_head: &Hash) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            let mut heads = write_txn.open_table(HEADS)?;
            let mut keys = Vec::new();
            for result in heads.iter()? {
                let (hash, _) = result?;
                keys.push(*hash.value());
            }
            for key in keys {
                heads.remove(&key)?;
            }
            heads.insert(&new_head.0, 1)?;
        }
        write_txn.commit()?;
        Ok(())
    }
}
