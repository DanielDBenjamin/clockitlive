use sqlx::SqlitePool;

/// Run database migrations
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    println!("🔄 Running database migrations...");

    // Set locking to false to ignore checksum mismatches
    sqlx::migrate!("./migrations").set_locking(false).run(pool).await?;

    println!("✅ Database migrations completed successfully!");
    Ok(())
}

/// Verify that the database tables exist and are accessible
pub async fn test_database_structure(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    println!("🧪 Testing database structure...");

    // Test that we can query each table (should return 0 rows initially)
    let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    let module_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM modules")
        .fetch_one(pool)
        .await?;

    let class_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM classes")
        .fetch_one(pool)
        .await?;

    println!("📊 Database state:");
    println!("   👥 Users: {}", user_count.0);
    println!("   📚 Modules: {}", module_count.0);
    println!("   🎓 Classes: {}", class_count.0);
    println!("✅ Database structure test successful!");

    Ok(())
}
