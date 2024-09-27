use super::{CacheRepoKey, CacheRepoValue};
use crate::Result;
use camino::Utf8Path;
use eyre::Context;
use redb::{Database, Table, TableDefinition};
use std::sync::Arc;

const TABLE: TableDefinition<CacheRepoKey, CacheRepoValue> = TableDefinition::new("data");

#[derive(Clone)]
struct Db {
    db: Arc<Database>,
}

impl Db {
    #[instrument(level = "info")]
    pub fn new(path: &Utf8Path) -> Result<Db> {
        let db = Database::create(path).with_context(|| "无法创建或者打开 redb 数据库文件")?;
        let db = Arc::new(db);
        Ok(Db { db })
    }

    pub fn get(&self, key: &CacheRepoKey) -> Result<Option<CacheRepoValue>> {
        let table = self.db.begin_read()?.open_table(TABLE)?;
        Ok(table.get(key)?.map(|guard| guard.value()))
    }

    fn write(
        &self,
        f: impl for<'a> FnOnce(&mut Table<'a, CacheRepoKey, CacheRepoValue>) -> Result<()>,
    ) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        f(&mut write_txn.open_table(TABLE)?)?;
        write_txn.commit()?;
        Ok(())
    }

    pub fn set(&self, key: &CacheRepoKey, value: CacheRepoValue) -> Result<()> {
        self.write(|table| {
            table.insert(key, value)?;
            Ok(())
        })
    }

    pub fn set_or_replace(
        &self,
        key: &CacheRepoKey,
        f: impl FnOnce(Option<CacheRepoValue>) -> Result<CacheRepoValue>,
    ) -> Result<()> {
        self.write(|table| {
            let opt_value = table.remove(key)?.map(|guard| guard.value());
            let mut value = f(opt_value)?;
            value.update_unix_timestamp();
            table.insert(key, value)?;
            Ok(())
        })
    }
}

#[test]
fn db() -> crate::Result<()> {
    let (key, value) = super::types::new_cache();

    let db = Database::builder().create_with_backend(redb::backends::InMemoryBackend::new())?;
    let db = Db { db: Arc::new(db) };

    db.set_or_replace(&key, move |opt| {
        assert!(opt.is_none());
        Ok(value)
    })?;

    db.set_or_replace(&key, move |opt| {
        let value = opt.unwrap();
        dbg!(&value);
        Ok(value)
    })?;

    Ok(())
}
