#[macro_use]
extern crate lazy_static;

use database::utils::{Command, UserInfo};
use sqlx::sqlite::SqlitePool;
use std::env;
use structopt::StructOpt;

mod backend;
mod config;
mod database;
mod node;

use backend::master;

// use crate::node::anvil;
use crate::database::utils::Args;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::from_args_safe()?;
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;
    let db = Sqlite::new(pool);

    match args.cmd {
        Some(Command::Start {}) => {
            println!("starting the master server...");
            master::start_server().await.unwrap();
            println!("server started");
        }
        Some(Command::Reset {}) => {
            println!("Resetting the fucking db");
            db.reset_db().await?;
        }
        _ => (),
        /*Some(Command::Add { description }) => {
            println!("Adding new todo with description '{}'", &description);
            let todo_id = db.add_todo(description).await?;
            println!("Added new todo with id {}", todo_id);
        }
        Some(Command::Done { id }) => {
            println!("Marking todo {} as done", id);
            if db.complete_todo(id).await? {
                println!("Todo {} is marked as done", id);
            } else {
                println!("Invalid id {}", id);
            }
        }
        None => {
            println!("Printing list of all todos");
            db.list_todos().await?;
        }*/
    }

    Ok(())
}

pub struct Sqlite {
    pub pool: SqlitePool,
}

impl Sqlite {
    fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    async fn reset_db(&self) -> anyhow::Result<()> {
        /*sqlx::query!(
                r#"
        DROP DATABASE huffpill
                "#
            )
            .fetch_all(&self.pool)
            .await?;*/

        sqlx::query!(
            r#"
    CREATE TABLE IF NOT EXISTS challenges (
            name TEXT PRIMARY KEY,
            difficulty INTEGER,
            solves INTEGER,
            kind VARCHAR(3) 
            )
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(())
    }

    /*async fn add_todo(&self, description: String) -> anyhow::Result<i64> {
            let mut conn = self.pool.acquire().await?;

            // Insert the task, then obtain the ID of this row
            let id = sqlx::query!(
                r#"
    INSERT INTO todos ( description )
    VALUES ( ?1 )
            "#,
                description
            )
            .execute(&mut conn)
            .await?
            .last_insert_rowid();

            Ok(id)
        }

        async fn complete_todo(&self, id: i64) -> anyhow::Result<bool> {
            let rows_affected = sqlx::query!(
                r#"
    UPDATE todos
    SET done = TRUE
    WHERE id = ?1
            "#,
                id
            )
            .execute(&self.pool)
            .await?
            .rows_affected();

            Ok(rows_affected > 0)
        }

        async fn list_todos(&self) -> anyhow::Result<()> {
            let recs = sqlx::query!(
                r#"
    SELECT id, description, done
    FROM todos
    ORDER BY id
            "#
            )
            .fetch_all(&self.pool)
            .await?;

            for rec in recs {
                println!(
                    "- [{}] {}: {}",
                    if rec.done { "x" } else { " " },
                    rec.id,
                    &rec.description,
                );
            }

            Ok(())
        }*/

    async fn get_user_info(&self, key: String) -> Option<UserInfo> {
        sqlx::query_as!(
            UserInfo,
            r#"
SELECT name, port_in, port_out
FROM users
        "#
        )
        .fetch_optional(&self.pool)
        .await
        .ok()?
    }
}
