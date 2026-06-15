use crate::error::Result;
use crate::core::storage::repository::*;
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
        let bookmark = table.get(Self::CURRENT_BOOKMARK_KEY)?.map(|g| g.value().to_string());
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
}
