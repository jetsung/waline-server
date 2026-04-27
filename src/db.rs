use sea_orm::Database;
use tracing::info;

pub type Db = sea_orm::DatabaseConnection;

pub async fn connect(database_url: &str) -> Result<Db, sea_orm::DbErr> {
    let db = Database::connect(database_url).await?;
    info!("Database connected: {}", mask_url(database_url));
    Ok(db)
}

pub async fn migrate(db: &Db, database_url: &str) -> Result<(), sea_orm::DbErr> {
    if database_url.starts_with("sqlite") {
        migrate_sqlite(db).await
    } else if database_url.starts_with("mysql") || database_url.starts_with("mariadb") {
        migrate_mysql(db).await
    } else {
        migrate_postgres(db).await
    }
}

async fn migrate_sqlite(db: &Db) -> Result<(), sea_orm::DbErr> {
    use sea_orm::ConnectionTrait;

    db.execute_unprepared(
        r#"CREATE TABLE IF NOT EXISTS "wl_Comment" (
            "id" INTEGER PRIMARY KEY AUTOINCREMENT,
            "user_id" INTEGER,
            "comment" TEXT,
            "insertedAt" TEXT DEFAULT (datetime('now','localtime')),
            "ip" TEXT,
            "link" TEXT,
            "mail" TEXT,
            "nick" TEXT,
            "rid" INTEGER,
            "pid" INTEGER,
            "sticky" NUMERIC,
            "status" TEXT NOT NULL DEFAULT '',
            "like" INTEGER,
            "ua" TEXT,
            "url" TEXT,
            "createdAt" TEXT DEFAULT (datetime('now','localtime')),
            "updatedAt" TEXT DEFAULT (datetime('now','localtime'))
        )"#,
    )
    .await?;

    db.execute_unprepared(
        r#"CREATE TABLE IF NOT EXISTS "wl_Counter" (
            "id" INTEGER PRIMARY KEY AUTOINCREMENT,
            "time" INTEGER,
            "reaction0" INTEGER,
            "reaction1" INTEGER,
            "reaction2" INTEGER,
            "reaction3" INTEGER,
            "reaction4" INTEGER,
            "reaction5" INTEGER,
            "reaction6" INTEGER,
            "reaction7" INTEGER,
            "reaction8" INTEGER,
            "url" TEXT,
            "createdAt" TEXT DEFAULT (datetime('now','localtime')),
            "updatedAt" TEXT DEFAULT (datetime('now','localtime'))
        )"#,
    )
    .await?;

    db.execute_unprepared(
        r#"CREATE TABLE IF NOT EXISTS "wl_Users" (
            "id" INTEGER PRIMARY KEY AUTOINCREMENT,
            "display_name" TEXT NOT NULL DEFAULT '',
            "email" TEXT NOT NULL DEFAULT '',
            "password" TEXT NOT NULL DEFAULT '',
            "type" TEXT NOT NULL DEFAULT '',
            "label" TEXT,
            "github" TEXT,
            "twitter" TEXT,
            "facebook" TEXT,
            "google" TEXT,
            "weibo" TEXT,
            "qq" TEXT,
            "oidc" TEXT,
            "huawei" TEXT,
            "2fa" TEXT,
            "avatar" TEXT,
            "url" TEXT,
            "createdAt" TEXT DEFAULT (datetime('now','localtime')),
            "updatedAt" TEXT DEFAULT (datetime('now','localtime'))
        )"#,
    )
    .await?;

    Ok(())
}

async fn migrate_mysql(db: &Db) -> Result<(), sea_orm::DbErr> {
    use sea_orm::ConnectionTrait;

    db.execute_unprepared(
        r#"CREATE TABLE IF NOT EXISTS `wl_Comment` (
            `id` int(11) unsigned NOT NULL AUTO_INCREMENT,
            `user_id` int(11) DEFAULT NULL,
            `comment` text,
            `insertedAt` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
            `ip` varchar(100) DEFAULT '',
            `link` varchar(255) DEFAULT NULL,
            `mail` varchar(255) DEFAULT NULL,
            `nick` varchar(255) DEFAULT NULL,
            `pid` int(11) DEFAULT NULL,
            `rid` int(11) DEFAULT NULL,
            `sticky` boolean DEFAULT NULL,
            `status` varchar(50) NOT NULL DEFAULT '',
            `like` int(11) DEFAULT NULL,
            `ua` text,
            `url` varchar(255) DEFAULT NULL,
            `createdAt` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
            `updatedAt` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"#,
    )
    .await?;

    db.execute_unprepared(
        r#"CREATE TABLE IF NOT EXISTS `wl_Counter` (
            `id` int(11) unsigned NOT NULL AUTO_INCREMENT,
            `time` int(11) DEFAULT NULL,
            `reaction0` int(11) DEFAULT NULL,
            `reaction1` int(11) DEFAULT NULL,
            `reaction2` int(11) DEFAULT NULL,
            `reaction3` int(11) DEFAULT NULL,
            `reaction4` int(11) DEFAULT NULL,
            `reaction5` int(11) DEFAULT NULL,
            `reaction6` int(11) DEFAULT NULL,
            `reaction7` int(11) DEFAULT NULL,
            `reaction8` int(11) DEFAULT NULL,
            `url` varchar(255) NOT NULL DEFAULT '',
            `createdAt` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
            `updatedAt` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"#,
    )
    .await?;

    db.execute_unprepared(
        r#"CREATE TABLE IF NOT EXISTS `wl_Users` (
            `id` int(11) unsigned NOT NULL AUTO_INCREMENT,
            `display_name` varchar(255) NOT NULL DEFAULT '',
            `email` varchar(255) NOT NULL DEFAULT '',
            `password` varchar(255) NOT NULL DEFAULT '',
            `type` varchar(50) NOT NULL DEFAULT '',
            `label` varchar(255) DEFAULT NULL,
            `url` varchar(255) DEFAULT NULL,
            `avatar` varchar(255) DEFAULT NULL,
            `github` varchar(255) DEFAULT NULL,
            `twitter` varchar(255) DEFAULT NULL,
            `facebook` varchar(255) DEFAULT NULL,
            `google` varchar(255) DEFAULT NULL,
            `weibo` varchar(255) DEFAULT NULL,
            `qq` varchar(255) DEFAULT NULL,
            `oidc` varchar(255) DEFAULT NULL,
            `huawei` varchar(255) DEFAULT NULL,
            `2fa` varchar(32) DEFAULT NULL,
            `createdAt` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
            `updatedAt` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            UNIQUE KEY `idx_user_email` (`email`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"#,
    )
    .await?;

    Ok(())
}

async fn migrate_postgres(db: &Db) -> Result<(), sea_orm::DbErr> {
    use sea_orm::ConnectionTrait;

    db.execute_unprepared(
        r#"CREATE TABLE IF NOT EXISTS "wl_Comment" (
            id SERIAL PRIMARY KEY,
            user_id INTEGER,
            comment TEXT,
            "insertedAt" TIMESTAMPTZ DEFAULT NOW(),
            ip VARCHAR(100),
            link VARCHAR(255),
            mail VARCHAR(255),
            nick VARCHAR(255),
            pid INTEGER,
            rid INTEGER,
            sticky BOOLEAN,
            status VARCHAR(50) NOT NULL DEFAULT '',
            "like" INTEGER,
            ua TEXT,
            url VARCHAR(255),
            "createdAt" TIMESTAMPTZ DEFAULT NOW(),
            "updatedAt" TIMESTAMPTZ DEFAULT NOW()
        )"#,
    )
    .await?;

    db.execute_unprepared(
        r#"CREATE TABLE IF NOT EXISTS "wl_Counter" (
            id SERIAL PRIMARY KEY,
            time INTEGER,
            reaction0 INTEGER,
            reaction1 INTEGER,
            reaction2 INTEGER,
            reaction3 INTEGER,
            reaction4 INTEGER,
            reaction5 INTEGER,
            reaction6 INTEGER,
            reaction7 INTEGER,
            reaction8 INTEGER,
            url VARCHAR(255),
            "createdAt" TIMESTAMPTZ DEFAULT NOW(),
            "updatedAt" TIMESTAMPTZ DEFAULT NOW()
        )"#,
    )
    .await?;

    db.execute_unprepared(
        r#"CREATE TABLE IF NOT EXISTS "wl_Users" (
            id SERIAL PRIMARY KEY,
            display_name VARCHAR(255) NOT NULL DEFAULT '',
            email VARCHAR(255) NOT NULL DEFAULT '',
            password VARCHAR(255) NOT NULL DEFAULT '',
            type VARCHAR(50) NOT NULL DEFAULT '',
            label VARCHAR(255),
            url VARCHAR(255),
            avatar VARCHAR(255),
            github VARCHAR(255),
            twitter VARCHAR(255),
            facebook VARCHAR(255),
            google VARCHAR(255),
            weibo VARCHAR(255),
            qq VARCHAR(255),
            oidc VARCHAR(255),
            huawei VARCHAR(255),
            "2fa" VARCHAR(32),
            "createdAt" TIMESTAMPTZ DEFAULT NOW(),
            "updatedAt" TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE (email)
        )"#,
    )
    .await?;

    Ok(())
}

fn mask_url(url: &str) -> String {
    if let Some(at) = url.find('@') {
        if let Some(end) = url.find("://") {
            return format!("{}***{}", &url[..end + 3], &url[at..]);
        }
    }
    url.to_string()
}
