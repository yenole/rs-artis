mod types;

pub mod migrator;
pub use migrator::{ArtisMigrator, DriverMigrator};
pub use types::{Adjust, ColumeMeta, IndexMeta, Mapping, TableMeta};

#[cfg(feature = "sqlite")]
mod migrator_sqlite;

#[cfg(feature = "sqlite")]
pub use migrator_sqlite::SqliteMigrator;

#[cfg(feature = "mysql")]
mod migrator_mysql;

#[cfg(feature = "mysql")]
pub use migrator_mysql::MysqlMigrator;

#[cfg(feature = "postgres")]
mod migrator_postgres;

#[cfg(feature = "postgres")]
pub use migrator_postgres::PostgresMigrator;
