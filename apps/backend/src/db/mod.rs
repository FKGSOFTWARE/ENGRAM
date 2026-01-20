//! Database Module
//!
//! Handles database connection, migrations, and schema management.

use sqlx::SqlitePool;

/// Migration files embedded at compile time
const MIGRATIONS: &[(&str, &str)] = &[
    (
        "000_migrations_table",
        include_str!("migrations/000_migrations_table.sql"),
    ),
    (
        "001_create_sources",
        include_str!("migrations/001_create_sources.sql"),
    ),
    (
        "002_create_cards",
        include_str!("migrations/002_create_cards.sql"),
    ),
    (
        "003_create_reviews",
        include_str!("migrations/003_create_reviews.sql"),
    ),
];

/// Run all pending database migrations
pub async fn migrate(pool: &SqlitePool) -> anyhow::Result<()> {
    tracing::info!("Running database migrations...");

    // First, ensure the migrations table exists
    let (_, migrations_sql) = MIGRATIONS[0];
    sqlx::query(migrations_sql).execute(pool).await?;

    // Get list of already applied migrations
    let applied: Vec<String> = sqlx::query_scalar("SELECT name FROM _migrations")
        .fetch_all(pool)
        .await
        .unwrap_or_default();

    // Run each migration that hasn't been applied
    let mut applied_count = 0;
    for (name, sql) in MIGRATIONS.iter().skip(1) {
        if applied.contains(&name.to_string()) {
            tracing::debug!("Migration {} already applied, skipping", name);
            continue;
        }

        tracing::info!("Applying migration: {}", name);

        // Execute migration SQL (may contain multiple statements)
        for statement in sql.split(';').filter(|s| !s.trim().is_empty()) {
            sqlx::query(statement).execute(pool).await?;
        }

        // Record migration as applied
        sqlx::query("INSERT INTO _migrations (name) VALUES (?)")
            .bind(*name)
            .execute(pool)
            .await?;

        applied_count += 1;
    }

    if applied_count > 0 {
        tracing::info!("Applied {} new migration(s)", applied_count);
    } else {
        tracing::info!("Database schema is up to date");
    }

    Ok(())
}

/// Check if the database needs migrations
#[allow(dead_code)]
pub async fn needs_migration(pool: &SqlitePool) -> bool {
    // Check if migrations table exists
    let table_exists: Option<i32> = sqlx::query_scalar(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='_migrations'",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    if table_exists.is_none() {
        return true;
    }

    // Check if all migrations are applied
    let applied_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _migrations")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    // -1 because we don't count the migrations table creation
    applied_count < (MIGRATIONS.len() as i64 - 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn test_migrations() {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        // First run should apply all migrations
        let result = migrate(&pool).await;
        assert!(result.is_ok());

        // Second run should be a no-op
        let result = migrate(&pool).await;
        assert!(result.is_ok());

        // Verify tables exist
        let sources_exists: Option<i32> = sqlx::query_scalar(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='sources'",
        )
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert!(sources_exists.is_some());

        let cards_exists: Option<i32> = sqlx::query_scalar(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='cards'",
        )
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert!(cards_exists.is_some());

        let reviews_exists: Option<i32> = sqlx::query_scalar(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='reviews'",
        )
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert!(reviews_exists.is_some());
    }
}
