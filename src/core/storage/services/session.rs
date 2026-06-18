use crate::core::storage::repository::*;
use crate::error::Result;
use redb::ReadableTable;

pub struct SessionService<'a> {
    pub(crate) repo: &'a Repository,
}

impl<'a> SessionService<'a> {
    const CURRENT_BOOKMARK_KEY: &'static str = "current_bookmark";

    pub fn set_current_bookmark(&self, name: &str) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            let mut table = write_txn.open_table(SESSION)?;
            table.insert(Self::CURRENT_BOOKMARK_KEY, name)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_current_bookmark(&self) -> Result<Option<String>> {
        let read_txn = self.repo.db.begin_read()?;
        let table = read_txn.open_table(SESSION)?;
        let bookmark = table
            .get(Self::CURRENT_BOOKMARK_KEY)?
            .map(|g| g.value().to_string());
        Ok(bookmark)
    }

    pub fn clear_current_bookmark(&self) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            let mut table = write_txn.open_table(SESSION)?;
            table.remove(Self::CURRENT_BOOKMARK_KEY)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    const MERGE_HEAD_KEY: &'static str = "merge_head";

    pub fn set_merge_head(&self, hash: &crate::core::hash::Hash) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            let mut table = write_txn.open_table(SESSION)?;
            table.insert(Self::MERGE_HEAD_KEY, hash.to_string().as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_merge_head(&self) -> Result<Option<crate::core::hash::Hash>> {
        let read_txn = self.repo.db.begin_read()?;
        let table = read_txn.open_table(SESSION)?;
        let hash_val = table
            .get(Self::MERGE_HEAD_KEY)?
            .map(|g| g.value().to_string());
        if let Some(hash_str) = hash_val {
            let hash = crate::core::hash::Hash::from_hex(&hash_str)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            Ok(Some(hash))
        } else {
            Ok(None)
        }
    }

    pub fn clear_merge_head(&self) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            let mut table = write_txn.open_table(SESSION)?;
            table.remove(Self::MERGE_HEAD_KEY)?;
        }
        write_txn.commit()?;
        Ok(())
    }
}
