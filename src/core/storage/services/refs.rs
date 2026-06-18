use crate::error::Result;
use crate::core::hash::Hash;
use crate::core::storage::repository::*;
use redb::ReadableTable;

pub struct RefService<'a> {
    pub(crate) repo: &'a Repository,
}

impl<'a> RefService<'a> {
    pub fn set_ref(&self, name: &str, hash: &Hash) -> Result<Option<Hash>> {
        let write_txn = self.repo.db.begin_write()?;
        let old_hash = {
            let mut table = write_txn.open_table(REFS)?;
            let old = table.get(name)?.map(|g| Hash(*g.value()));
            table.insert(name, &hash.0)?;
            old
        };
        write_txn.commit()?;
        Ok(old_hash)
    }

    pub fn get_ref(&self, name: &str) -> Result<Option<Hash>> {
        let read_txn = self.repo.db.begin_read()?;
        let table = read_txn.open_table(REFS)?;
        let hash = table.get(name)?.map(|guard| Hash(*guard.value()));
        Ok(hash)
    }

    pub fn list_refs(&self) -> Result<Vec<(String, Hash)>> {
        let read_txn = self.repo.db.begin_read()?;
        let table = read_txn.open_table(REFS)?;
        let mut entries = Vec::new();
        for result in table.iter()? {
            let (name, hash) = result?;
            entries.push((name.value().to_string(), Hash(*hash.value())));
        }
        Ok(entries)
    }

    pub fn delete_ref(&self, name: &str) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            let mut table = write_txn.open_table(REFS)?;
            table.remove(name)?;
        }
        write_txn.commit()?;
        Ok(())
    }
}
