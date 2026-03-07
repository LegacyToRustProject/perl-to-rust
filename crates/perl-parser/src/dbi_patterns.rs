//! DBI → SQLx conversion patterns
//! Maps common Perl DBI/DBD usage patterns to their SQLx equivalents.
//! This module provides:
//! 1. Pattern recognition for DBI calls in Perl source
//! 2. Generated Rust/SQLx code templates
//! 3. Connection string conversion (DSN → DATABASE_URL)

use regex::Regex;

/// A recognized DBI usage pattern with its SQLx equivalent.
#[derive(Debug, Clone, PartialEq)]
pub struct DbiPattern {
    /// Descriptive name of the pattern.
    pub name: &'static str,
    /// The Perl DBI code pattern (for display/docs).
    pub perl_pattern: &'static str,
    /// The Rust/SQLx equivalent code.
    pub rust_equivalent: &'static str,
    /// Required Cargo.toml dependencies.
    pub dependencies: &'static [&'static str],
    /// Category for grouping.
    pub category: DbiCategory,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DbiCategory {
    Connection,
    Query,
    Execute,
    Fetch,
    Transaction,
    Metadata,
    Error,
    Utility,
}

/// Complete DBI → SQLx pattern library (30+ patterns).
pub fn all_patterns() -> Vec<DbiPattern> {
    vec![
        // ── CONNECTION ────────────────────────────────────────────────
        DbiPattern {
            name: "connect_mysql",
            perl_pattern: r#"DBI->connect("dbi:mysql:database=$db", $user, $pass)"#,
            rust_equivalent: r#"sqlx::MySqlPool::connect(&database_url).await?"#,
            dependencies: &[
                "sqlx = { version = \"0.8\", features = [\"mysql\", \"runtime-tokio\"] }",
            ],
            category: DbiCategory::Connection,
        },
        DbiPattern {
            name: "connect_postgres",
            perl_pattern: r#"DBI->connect("dbi:Pg:dbname=$db;host=$host", $user, $pass)"#,
            rust_equivalent: r#"sqlx::PgPool::connect(&database_url).await?"#,
            dependencies: &[
                "sqlx = { version = \"0.8\", features = [\"postgres\", \"runtime-tokio\"] }",
            ],
            category: DbiCategory::Connection,
        },
        DbiPattern {
            name: "connect_sqlite",
            perl_pattern: r#"DBI->connect("dbi:SQLite:dbname=$file")"#,
            rust_equivalent: r#"sqlx::SqlitePool::connect(&format!("sqlite:{}", file)).await?"#,
            dependencies: &[
                "sqlx = { version = \"0.8\", features = [\"sqlite\", \"runtime-tokio\"] }",
            ],
            category: DbiCategory::Connection,
        },
        DbiPattern {
            name: "connect_with_attrs",
            perl_pattern: r#"DBI->connect($dsn, $user, $pass, { RaiseError => 1, AutoCommit => 1 })"#,
            rust_equivalent: r#"sqlx::Pool::connect_with(
    sqlx::pool::PoolOptions::new()
        .max_connections(5)
        .connect_lazy(&database_url)?
)"#,
            dependencies: &["sqlx = { version = \"0.8\", features = [\"runtime-tokio\"] }"],
            category: DbiCategory::Connection,
        },
        DbiPattern {
            name: "disconnect",
            perl_pattern: r#"$dbh->disconnect()"#,
            rust_equivalent: r#"pool.close().await  // Pool drops when out of scope"#,
            dependencies: &[],
            category: DbiCategory::Connection,
        },
        // ── QUERY / PREPARE ──────────────────────────────────────────
        DbiPattern {
            name: "prepare",
            perl_pattern: r#"my $sth = $dbh->prepare("SELECT * FROM users WHERE id = ?")"#,
            rust_equivalent: r#"sqlx::query("SELECT * FROM users WHERE id = ?")
    .bind(id)"#,
            dependencies: &[],
            category: DbiCategory::Query,
        },
        DbiPattern {
            name: "prepare_named",
            perl_pattern: r#"$dbh->prepare("SELECT * FROM users WHERE id = :id")"#,
            rust_equivalent: r#"sqlx::query("SELECT * FROM users WHERE id = $1")
    .bind(id)"#, // PostgreSQL uses $1, MySQL uses ?
            dependencies: &[],
            category: DbiCategory::Query,
        },
        DbiPattern {
            name: "prepare_as",
            perl_pattern: r#"$dbh->prepare("SELECT id, name FROM users")"#,
            rust_equivalent: r#"sqlx::query_as::<_, User>("SELECT id, name FROM users")"#,
            dependencies: &[],
            category: DbiCategory::Query,
        },
        DbiPattern {
            name: "do_insert",
            perl_pattern: r#"$dbh->do("INSERT INTO logs (msg) VALUES (?)", undef, $msg)"#,
            rust_equivalent: r#"sqlx::query("INSERT INTO logs (msg) VALUES (?)")
    .bind(msg)
    .execute(&pool)
    .await?"#,
            dependencies: &[],
            category: DbiCategory::Execute,
        },
        DbiPattern {
            name: "do_update",
            perl_pattern: r#"$dbh->do("UPDATE users SET name = ? WHERE id = ?", undef, $name, $id)"#,
            rust_equivalent: r#"sqlx::query("UPDATE users SET name = ? WHERE id = ?")
    .bind(name)
    .bind(id)
    .execute(&pool)
    .await?"#,
            dependencies: &[],
            category: DbiCategory::Execute,
        },
        DbiPattern {
            name: "do_delete",
            perl_pattern: r#"$dbh->do("DELETE FROM sessions WHERE expires < ?", undef, $now)"#,
            rust_equivalent: r#"sqlx::query("DELETE FROM sessions WHERE expires < ?")
    .bind(now)
    .execute(&pool)
    .await?"#,
            dependencies: &[],
            category: DbiCategory::Execute,
        },
        // ── EXECUTE ───────────────────────────────────────────────────
        DbiPattern {
            name: "execute",
            perl_pattern: r#"$sth->execute($id)"#,
            rust_equivalent: r#".execute(&pool).await?"#,
            dependencies: &[],
            category: DbiCategory::Execute,
        },
        DbiPattern {
            name: "execute_multiple_params",
            perl_pattern: r#"$sth->execute($name, $age, $email)"#,
            rust_equivalent: r#".bind(name).bind(age).bind(email).execute(&pool).await?"#,
            dependencies: &[],
            category: DbiCategory::Execute,
        },
        DbiPattern {
            name: "rows_affected",
            perl_pattern: r#"my $rows = $sth->rows()"#,
            rust_equivalent: r#"let result = query.execute(&pool).await?;
let rows = result.rows_affected();"#,
            dependencies: &[],
            category: DbiCategory::Execute,
        },
        DbiPattern {
            name: "last_insert_id",
            perl_pattern: r#"my $id = $dbh->last_insert_id(undef, undef, "users", "id")"#,
            rust_equivalent: r#"let result = sqlx::query("INSERT ...").execute(&pool).await?;
let id = result.last_insert_id();"#,
            dependencies: &[],
            category: DbiCategory::Execute,
        },
        // ── FETCH ─────────────────────────────────────────────────────
        DbiPattern {
            name: "fetchrow_hashref",
            perl_pattern: r#"my $row = $sth->fetchrow_hashref()"#,
            rust_equivalent: r#"let row = sqlx::query_as::<_, User>(sql)
    .fetch_one(&pool).await?;"#,
            dependencies: &[],
            category: DbiCategory::Fetch,
        },
        DbiPattern {
            name: "fetchrow_arrayref",
            perl_pattern: r#"my $row = $sth->fetchrow_arrayref()"#,
            rust_equivalent: r#"let row = sqlx::query(sql).fetch_one(&pool).await?;
let val: i64 = row.get(0);"#,
            dependencies: &[],
            category: DbiCategory::Fetch,
        },
        DbiPattern {
            name: "fetchrow_array",
            perl_pattern: r#"my @row = $sth->fetchrow_array()"#,
            rust_equivalent: r#"let row = sqlx::query(sql).fetch_one(&pool).await?;
let (id, name): (i64, String) = (row.get(0), row.get(1));"#,
            dependencies: &[],
            category: DbiCategory::Fetch,
        },
        DbiPattern {
            name: "fetchall_arrayref",
            perl_pattern: r#"my $rows = $sth->fetchall_arrayref({})"#,
            rust_equivalent: r#"let rows = sqlx::query_as::<_, User>(sql)
    .fetch_all(&pool).await?;"#,
            dependencies: &[],
            category: DbiCategory::Fetch,
        },
        DbiPattern {
            name: "fetch_optional",
            perl_pattern: r#"if (my $row = $sth->fetchrow_hashref()) { ... }"#,
            rust_equivalent: r#"if let Some(row) = sqlx::query_as::<_, User>(sql)
    .fetch_optional(&pool).await? {
    // use row
}"#,
            dependencies: &[],
            category: DbiCategory::Fetch,
        },
        DbiPattern {
            name: "fetch_loop",
            perl_pattern: r#"while (my $row = $sth->fetchrow_hashref()) { process($row); }"#,
            rust_equivalent: r#"let mut rows = sqlx::query_as::<_, User>(sql).fetch(&pool);
while let Some(row) = rows.try_next().await? {
    process(row);
}"#,
            dependencies: &["futures = \"0.3\""],
            category: DbiCategory::Fetch,
        },
        DbiPattern {
            name: "selectrow_hashref",
            perl_pattern: r#"my $row = $dbh->selectrow_hashref("SELECT * FROM users WHERE id = ?", undef, $id)"#,
            rust_equivalent: r#"let row: Option<User> = sqlx::query_as("SELECT * FROM users WHERE id = ?")
    .bind(id)
    .fetch_optional(&pool).await?;"#,
            dependencies: &[],
            category: DbiCategory::Fetch,
        },
        DbiPattern {
            name: "selectall_arrayref",
            perl_pattern: r#"my $rows = $dbh->selectall_arrayref($sql, { Slice => {} })"#,
            rust_equivalent: r#"let rows: Vec<User> = sqlx::query_as($sql)
    .fetch_all(&pool).await?;"#,
            dependencies: &[],
            category: DbiCategory::Fetch,
        },
        DbiPattern {
            name: "selectcol_arrayref",
            perl_pattern: r#"my $ids = $dbh->selectcol_arrayref("SELECT id FROM users")"#,
            rust_equivalent: r#"let ids: Vec<i64> = sqlx::query_scalar("SELECT id FROM users")
    .fetch_all(&pool).await?;"#,
            dependencies: &[],
            category: DbiCategory::Fetch,
        },
        // ── TRANSACTION ───────────────────────────────────────────────
        DbiPattern {
            name: "begin_work",
            perl_pattern: r#"$dbh->begin_work();"#,
            rust_equivalent: r#"let mut tx = pool.begin().await?;"#,
            dependencies: &[],
            category: DbiCategory::Transaction,
        },
        DbiPattern {
            name: "commit",
            perl_pattern: r#"$dbh->commit();"#,
            rust_equivalent: r#"tx.commit().await?;"#,
            dependencies: &[],
            category: DbiCategory::Transaction,
        },
        DbiPattern {
            name: "rollback",
            perl_pattern: r#"$dbh->rollback();"#,
            rust_equivalent: r#"tx.rollback().await?;"#,
            dependencies: &[],
            category: DbiCategory::Transaction,
        },
        DbiPattern {
            name: "transaction_block",
            perl_pattern: r#"eval {
    $dbh->begin_work();
    $dbh->do($sql1);
    $dbh->do($sql2);
    $dbh->commit();
};
if ($@) { $dbh->rollback(); }"#,
            rust_equivalent: r#"let mut tx = pool.begin().await?;
sqlx::query(sql1).execute(&mut *tx).await?;
sqlx::query(sql2).execute(&mut *tx).await?;
tx.commit().await?;
// tx.rollback() called automatically on Drop if not committed"#,
            dependencies: &[],
            category: DbiCategory::Transaction,
        },
        // ── METADATA ──────────────────────────────────────────────────
        DbiPattern {
            name: "column_names",
            perl_pattern: r#"my @cols = @{$sth->{NAME}}"#,
            rust_equivalent: r#"// Use sqlx::query_as with #[derive(FromRow)] struct
// Column names available at compile time via the derive macro"#,
            dependencies: &[],
            category: DbiCategory::Metadata,
        },
        DbiPattern {
            name: "tables",
            perl_pattern: r#"my @tables = $dbh->tables()"#,
            rust_equivalent: r#"// MySQL: SHOW TABLES  → sqlx::query_scalar("SHOW TABLES")
// PostgreSQL: SELECT tablename FROM pg_tables WHERE schemaname='public'
let tables: Vec<String> = sqlx::query_scalar("SHOW TABLES")
    .fetch_all(&pool).await?;"#,
            dependencies: &[],
            category: DbiCategory::Metadata,
        },
        DbiPattern {
            name: "quote",
            perl_pattern: r#"my $quoted = $dbh->quote($value)"#,
            rust_equivalent: r#"// Use parameterized queries — quoting is handled by SQLx automatically
// Never interpolate user input: use .bind(value) instead"#,
            dependencies: &[],
            category: DbiCategory::Utility,
        },
        // ── ERROR HANDLING ────────────────────────────────────────────
        DbiPattern {
            name: "errstr",
            perl_pattern: r#"die $dbh->errstr if $dbh->err;"#,
            rust_equivalent: r#"// SQLx uses Result<T, sqlx::Error> automatically
// match err { sqlx::Error::Database(e) => eprintln!("{}", e.message()) }"#,
            dependencies: &[],
            category: DbiCategory::Error,
        },
        DbiPattern {
            name: "raise_error",
            perl_pattern: r#"DBI->connect($dsn, $u, $p, { RaiseError => 1 })"#,
            rust_equivalent: r#"// RaiseError => 1 is the default behavior in SQLx (errors are Results)
// All SQLx operations return Result<T, sqlx::Error>"#,
            dependencies: &[],
            category: DbiCategory::Error,
        },
        // ── STRUCT DERIVE ─────────────────────────────────────────────
        DbiPattern {
            name: "from_row_derive",
            perl_pattern: r#"# Automatically mapping fetchrow_hashref to a struct"#,
            rust_equivalent: r#"#[derive(Debug, sqlx::FromRow)]
struct User {
    id: i64,
    name: String,
    email: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}"#,
            dependencies: &[
                "sqlx = { version = \"0.8\", features = [\"chrono\"] }",
                "chrono = { version = \"0.4\", features = [\"serde\"] }",
            ],
            category: DbiCategory::Utility,
        },
        DbiPattern {
            name: "query_builder",
            perl_pattern: r#"my $sql = "SELECT * FROM users WHERE 1=1";
$sql .= " AND age > ?" if $age;
$sql .= " AND name LIKE ?" if $name;"#,
            rust_equivalent: r#"let mut qb = sqlx::QueryBuilder::new("SELECT * FROM users WHERE 1=1");
if let Some(age) = age {
    qb.push(" AND age > ").push_bind(age);
}
if let Some(name) = name {
    qb.push(" AND name LIKE ").push_bind(format!("%{}%", name));
}
let rows = qb.build_query_as::<User>().fetch_all(&pool).await?;"#,
            dependencies: &[],
            category: DbiCategory::Query,
        },
    ]
}

/// Detect DBI patterns in Perl source code.
pub struct DbiDetector {
    connect_re: Regex,
    prepare_re: Regex,
    execute_re: Regex,
    fetch_re: Regex,
    do_re: Regex,
    transaction_re: Regex,
}

impl DbiDetector {
    pub fn new() -> Self {
        Self {
            connect_re: Regex::new(r#"DBI->connect\s*\("#).unwrap(),
            prepare_re: Regex::new(r#"\$\w+->prepare\s*\("#).unwrap(),
            execute_re: Regex::new(r#"\$\w+->execute\s*\("#).unwrap(),
            fetch_re: Regex::new(
                r#"\$\w+->fetch(?:row_(?:hashref|arrayref|array)|all_arrayref|all_hashref)\s*\("#,
            )
            .unwrap(),
            do_re: Regex::new(r#"\$\w+->do\s*\("#).unwrap(),
            transaction_re: Regex::new(r#"\$\w+->(?:begin_work|commit|rollback)\s*\("#).unwrap(),
        }
    }

    /// Count DBI usage patterns in Perl source.
    pub fn count_patterns(&self, source: &str) -> DbiUsageSummary {
        DbiUsageSummary {
            connects: self.connect_re.find_iter(source).count(),
            prepares: self.prepare_re.find_iter(source).count(),
            executes: self.execute_re.find_iter(source).count(),
            fetches: self.fetch_re.find_iter(source).count(),
            dos: self.do_re.find_iter(source).count(),
            transactions: self.transaction_re.find_iter(source).count(),
        }
    }

    /// Check if source contains any DBI usage.
    pub fn has_dbi(&self, source: &str) -> bool {
        self.connect_re.is_match(source)
            || self.prepare_re.is_match(source)
            || self.do_re.is_match(source)
    }
}

impl Default for DbiDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default)]
pub struct DbiUsageSummary {
    pub connects: usize,
    pub prepares: usize,
    pub executes: usize,
    pub fetches: usize,
    pub dos: usize,
    pub transactions: usize,
}

impl DbiUsageSummary {
    pub fn total(&self) -> usize {
        self.connects + self.prepares + self.executes + self.fetches + self.dos + self.transactions
    }
}

/// Convert a DBI DSN string to a DATABASE_URL format.
///
/// Perl DSN:   "dbi:mysql:database=mydb;host=localhost;port=3306"
/// SQLx URL:   "mysql://user:pass@localhost:3306/mydb"
pub fn dsn_to_database_url(dsn: &str, user: &str, pass: &str) -> Option<String> {
    let dsn_re = Regex::new(
        r#"dbi:(\w+):(?:(?:database|dbname)=([^;]+))?(?:;host=([^;]+))?(?:;port=(\d+))?"#,
    )
    .unwrap();

    if let Some(caps) = dsn_re.captures(dsn) {
        let driver = caps.get(1)?.as_str().to_lowercase();
        let database = caps.get(2).map(|m| m.as_str()).unwrap_or("mydb");
        let host = caps.get(3).map(|m| m.as_str()).unwrap_or("localhost");
        let port = caps.get(4).map(|m| m.as_str());

        let scheme = match driver.as_str() {
            "mysql" => "mysql",
            "pg" | "postgres" | "postgresql" => "postgres",
            "sqlite" => return Some(format!("sqlite:{}", database)),
            _ => return None,
        };

        let port_str = port.map(|p| format!(":{}", p)).unwrap_or_default();

        Some(format!(
            "{}://{}:{}@{}{}/{}",
            scheme, user, pass, host, port_str, database
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_patterns_count() {
        let patterns = all_patterns();
        assert!(
            patterns.len() >= 30,
            "Need at least 30 DBI patterns, got {}",
            patterns.len()
        );
    }

    #[test]
    fn test_patterns_have_rust_equivalent() {
        for p in all_patterns() {
            assert!(
                !p.rust_equivalent.is_empty(),
                "Pattern '{}' has empty rust_equivalent",
                p.name
            );
        }
    }

    #[test]
    fn test_all_categories_represented() {
        let patterns = all_patterns();
        let has_connection = patterns
            .iter()
            .any(|p| p.category == DbiCategory::Connection);
        let has_query = patterns.iter().any(|p| p.category == DbiCategory::Query);
        let has_fetch = patterns.iter().any(|p| p.category == DbiCategory::Fetch);
        let has_transaction = patterns
            .iter()
            .any(|p| p.category == DbiCategory::Transaction);
        assert!(has_connection);
        assert!(has_query);
        assert!(has_fetch);
        assert!(has_transaction);
    }

    #[test]
    fn test_detector_connect() {
        let detector = DbiDetector::new();
        let source = r#"my $dbh = DBI->connect("dbi:mysql:db=mydb", $user, $pass);"#;
        assert!(detector.has_dbi(source));
        let summary = detector.count_patterns(source);
        assert_eq!(summary.connects, 1);
    }

    #[test]
    fn test_detector_prepare_execute_fetch() {
        let detector = DbiDetector::new();
        let source = r#"
my $sth = $dbh->prepare("SELECT * FROM users WHERE id = ?");
$sth->execute($id);
my $row = $sth->fetchrow_hashref();
"#;
        let summary = detector.count_patterns(source);
        assert_eq!(summary.prepares, 1);
        assert_eq!(summary.executes, 1);
        assert_eq!(summary.fetches, 1);
        assert_eq!(summary.total(), 3);
    }

    #[test]
    fn test_detector_transaction() {
        let detector = DbiDetector::new();
        let source = r#"
$dbh->begin_work();
$dbh->do("INSERT INTO log VALUES (?)");
$dbh->commit();
"#;
        let summary = detector.count_patterns(source);
        assert_eq!(summary.transactions, 2); // begin_work + commit
        assert_eq!(summary.dos, 1);
    }

    #[test]
    fn test_dsn_to_database_url_mysql() {
        let url = dsn_to_database_url(
            "dbi:mysql:database=mydb;host=localhost;port=3306",
            "user",
            "pass",
        )
        .unwrap();
        assert_eq!(url, "mysql://user:pass@localhost:3306/mydb");
    }

    #[test]
    fn test_dsn_to_database_url_postgres() {
        let url =
            dsn_to_database_url("dbi:Pg:dbname=mydb;host=db.example.com", "user", "pass").unwrap();
        assert_eq!(url, "postgres://user:pass@db.example.com/mydb");
    }

    #[test]
    fn test_dsn_to_database_url_sqlite() {
        let url = dsn_to_database_url("dbi:SQLite:dbname=./mydb.sqlite", "user", "pass").unwrap();
        assert_eq!(url, "sqlite:./mydb.sqlite");
    }

    #[test]
    fn test_pattern_by_name() {
        let patterns = all_patterns();
        let connect = patterns.iter().find(|p| p.name == "connect_mysql").unwrap();
        assert!(connect.perl_pattern.contains("DBI->connect"));
        assert!(connect.rust_equivalent.contains("MySqlPool"));
    }

    #[test]
    fn test_transaction_block_pattern() {
        let patterns = all_patterns();
        let tx = patterns
            .iter()
            .find(|p| p.name == "transaction_block")
            .unwrap();
        assert!(tx.perl_pattern.contains("begin_work"));
        assert!(tx.rust_equivalent.contains("tx.commit"));
    }
}

#[cfg(test)]
mod extra_tests {
    use super::*;

    #[test]
    fn test_dsn_no_port() {
        let url = dsn_to_database_url("dbi:mysql:database=mydb;host=db.internal", "root", "s3cr3t")
            .unwrap();
        // Should not have a bare colon before the database name
        assert!(url.starts_with("mysql://root:s3cr3t@db.internal/mydb"));
    }

    #[test]
    fn test_dsn_unknown_driver() {
        let result = dsn_to_database_url("dbi:Oracle:service=mydb", "user", "pass");
        assert!(result.is_none());
    }

    #[test]
    fn test_detector_no_dbi_source() {
        let detector = DbiDetector::new();
        let source = "#!/usr/bin/perl\nprint \"Hello, world!\\n\";\n";
        assert!(!detector.has_dbi(source));
        assert_eq!(detector.count_patterns(source).total(), 0);
    }

    #[test]
    fn test_usage_summary_total() {
        let s = DbiUsageSummary {
            connects: 1,
            prepares: 3,
            executes: 3,
            fetches: 2,
            dos: 1,
            transactions: 2,
        };
        assert_eq!(s.total(), 12);
    }

    #[test]
    fn test_fetch_category_count() {
        let patterns = all_patterns();
        let fetch_count = patterns
            .iter()
            .filter(|p| p.category == DbiCategory::Fetch)
            .count();
        assert!(
            fetch_count >= 5,
            "expected >=5 fetch patterns, got {}",
            fetch_count
        );
    }

    #[test]
    fn test_connection_category_count() {
        let patterns = all_patterns();
        let conn_count = patterns
            .iter()
            .filter(|p| p.category == DbiCategory::Connection)
            .count();
        assert!(conn_count >= 3);
    }

    #[test]
    fn test_pattern_dependencies() {
        // connect_mysql pattern should have sqlx dependency
        let patterns = all_patterns();
        let mysql = patterns.iter().find(|p| p.name == "connect_mysql").unwrap();
        assert!(mysql.dependencies.iter().any(|d| d.contains("sqlx")));
    }
}
