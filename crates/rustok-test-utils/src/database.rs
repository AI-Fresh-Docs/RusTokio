//! # Database Test Utilities
//!
//! Provides database migration and setup utilities for testing.

use sea_orm::{ConnectionTrait, Database, DbConn, DbErr, ExecResult};
use sea_orm_migration::{MigratorTrait, Migrator};
use std::time::Duration;

/// Test database migrator
pub type TestMigrator = migration::Migrator;

/// Database test configuration
#[derive(Debug, Clone)]
pub struct TestDbConfig {
    /// Database URL for testing
    pub database_url: String,
    /// Whether to clean the database before migration
    pub clean_on_start: bool,
    /// Whether to run migrations on startup
    pub run_migrations: bool,
}

impl Default for TestDbConfig {
    fn default() -> Self {
        Self {
            database_url: std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/rustok_test".to_string()),
            clean_on_start: std::env::var("TEST_CLEAN_DB")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
            run_migrations: std::env::var("TEST_RUN_MIGRATIONS")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
        }
    }
}

/// Setup a test database connection
pub async fn setup_test_db(config: Option<TestDbConfig>) -> Result<DbConn, DbErr> {
    let config = config.unwrap_or_default();

    // Connect to the database
    let mut opt = sea_orm::ConnectOptions::new(config.database_url.clone());
    opt.max_connections(5)
        .connect_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(600))
        .sqlx_logging(false);

    let db = Database::connect(opt).await?;

    // Clean database if configured
    if config.clean_on_start {
        clean_test_db(&db).await?;
    }

    // Run migrations if configured
    if config.run_migrations {
        run_migrations(&db).await?;
    }

    Ok(db)
}

/// Clean the test database by dropping and recreating tables
pub async fn clean_test_db(db: &DbConn) -> Result<(), DbErr> {
    use sea_orm::{Statement, StatementBackend};

    // Get list of tables to drop
    let tables_to_drop = vec![
        "outbox_events",
        "build_releases",
        "builds",
        "event_versions",
        "inventory",
        "product_prices",
        "product_variants",
        "product_options",
        "products",
        "commerce_categories",
        "collections",
        "media",
        "meta",
        "tags",
        "categories",
        "node_translations",
        "nodes",
        "index_products",
        "index_content",
        "search_index",
        "tenant_locales",
        "tenant_modules",
        "sessions",
        "roles",
        "role_permissions",
        "user_roles",
        "tenants",
        "users",
    ];

    // Drop tables in correct order (respect foreign keys)
    for table in tables_to_drop {
        let stmt = Statement::from_string(
            db.get_database_backend(),
            format!("DROP TABLE IF EXISTS {} CASCADE", table),
        );

        // Ignore errors if table doesn't exist
        let _ = db.execute(stmt).await;
    }

    Ok(())
}

/// Run database migrations
pub async fn run_migrations(db: &DbConn) -> Result<(), DbErr> {
    TestMigrator::up(db, None).await?;
    Ok(())
}

/// Rollback all migrations
pub async fn rollback_migrations(db: &DbConn) -> Result<(), DbErr> {
    TestMigrator::down(db, None).await?;
    Ok(())
}

/// Reset the test database (clean + migrate)
pub async fn reset_test_db(db: &DbConn) -> Result<(), DbErr> {
    clean_test_db(db).await?;
    run_migrations(db).await?;
    Ok(())
}

/// Check if a table exists
pub async fn table_exists(db: &DbConn, table_name: &str) -> Result<bool, DbErr> {
    use sea_orm::{Statement, StatementBackend, FromQueryResult};

    #[derive(Debug, FromQueryResult)]
    struct TableExists {
        exists: bool,
    }

    let query = match db.get_database_backend() {
        sea_orm::DatabaseBackend::Postgres => {
            "SELECT EXISTS (
                SELECT FROM information_schema.tables
                WHERE table_name = $1
            ) as exists"
        }
        sea_orm::DatabaseBackend::Sqlite => {
            "SELECT EXISTS (
                SELECT 1 FROM sqlite_master WHERE type='table' AND name=?
            ) as exists"
        }
        sea_orm::DatabaseBackend::MySql => {
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables
                WHERE table_schema = DATABASE() AND table_name = ?
            ) as exists"
        }
        _ => {
            return Err(DbErr::Custom("Unsupported database backend".to_string()));
        }
    };

    let stmt = Statement::from_sql_and_values(
        db.get_database_backend(),
        query,
        vec![table_name.into()],
    );

    let result: TableExists = TableExists::find_by_statement(stmt)
        .one(db)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound)?;

    Ok(result.exists)
}

/// Get table row count
pub async fn count_table_rows(db: &DbConn, table_name: &str) -> Result<u64, DbErr> {
    use sea_orm::{Statement, StatementBackend, FromQueryResult};

    #[derive(Debug, FromQueryResult)]
    struct RowCount {
        count: Option<i64>,
    }

    let query = format!("SELECT COUNT(*) as count FROM {}", table_name);
    let stmt = Statement::from_string(db.get_database_backend(), query);

    let result: RowCount = RowCount::find_by_statement(stmt)
        .one(db)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound)?;

    Ok(result.count.unwrap_or(0) as u64)
}

/// Database test helper with automatic cleanup
pub struct TestDb {
    db: DbConn,
    config: TestDbConfig,
}

impl TestDb {
    /// Create a new test database with automatic cleanup
    pub async fn new() -> Result<Self, DbErr> {
        let config = TestDbConfig::default();
        let db = setup_test_db(Some(config.clone())).await?;
        Ok(Self { db, config })
    }

    /// Create a new test database with custom configuration
    pub async fn with_config(config: TestDbConfig) -> Result<Self, DbErr> {
        let db = setup_test_db(Some(config.clone())).await?;
        Ok(Self { db, config })
    }

    /// Get the database connection
    pub fn conn(&self) -> &DbConn {
        &self.db
    }

    /// Reset the database
    pub async fn reset(&self) -> Result<(), DbErr> {
        reset_test_db(&self.db).await
    }
}

impl Drop for TestDb {
    fn drop(&mut self) {
        // Clean up the database on drop if configured
        if self.config.clean_on_start {
            let db = self.db.clone();
            tokio::spawn(async move {
                let _ = clean_test_db(&db).await;
            });
        }
    }
}

/// Transaction helper for test isolation
pub async fn with_test_transaction<F, R>(db: &DbConn, f: F) -> Result<R, DbErr>
where
    F: FnOnce(&DbConn) -> Result<R, DbErr>,
{
    let txn = db.begin().await?;
    let result = match f(&txn) {
        Ok(r) => r,
        Err(e) => {
            txn.rollback().await?;
            return Err(e);
        }
    };
    txn.commit().await?;
    Ok(result)
}

/// Transaction helper that always rolls back (for test isolation)
pub async fn with_test_rollback<F, R>(db: &DbConn, f: F) -> Result<R, DbErr>
where
    F: FnOnce(&DbConn) -> Result<R, DbErr>,
{
    let txn = db.begin().await?;
    let result = f(&txn);
    txn.rollback().await?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires a test database
    async fn test_setup_test_db() {
        let config = TestDbConfig {
            database_url: std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/rustok_test".to_string()),
            clean_on_start: true,
            run_migrations: true,
        };

        let db = setup_test_db(Some(config)).await.expect("Failed to setup test db");

        // Check if migrations ran by checking for a table
        assert!(table_exists(&db, "tenants").await.unwrap());
    }

    #[tokio::test]
    #[ignore]
    async fn test_table_exists() {
        let db = TestDb::new().await.expect("Failed to create test db");
        assert!(table_exists(db.conn(), "tenants").await.unwrap());
        assert!(!table_exists(db.conn(), "nonexistent_table").await.unwrap());
    }

    #[tokio::test]
    #[ignore]
    async fn test_count_table_rows() {
        let db = TestDb::new().await.expect("Failed to create test db");
        let count = count_table_rows(db.conn(), "tenants").await.unwrap();
        assert_eq!(count, 0); // Empty after clean
    }

    #[tokio::test]
    #[ignore]
    async fn test_with_test_transaction() {
        let db = TestDb::new().await.expect("Failed to create test db");

        let result = with_test_transaction(db.conn(), |txn| {
            // Transaction would be committed here
            Ok(42)
        })
        .await
        .expect("Transaction failed");

        assert_eq!(result, 42);
    }

    #[tokio::test]
    #[ignore]
    async fn test_with_test_rollback() {
        let db = TestDb::new().await.expect("Failed to create test db");

        let result = with_test_rollback(db.conn(), |txn| {
            // Transaction will be rolled back
            Ok(42)
        })
        .await
        .expect("Rollback failed");

        assert_eq!(result, 42);
    }
}
