
use std::convert::TryFrom;
use async_trait::async_trait;

use deadpool_postgres::{Client, Config as DeadpoolConfig, Pool};
use tokio_postgres::{NoTls, row::Row};

use crate::time_provider::TimeProvider;
use crate::models::{MyError, Config, Storage, Board};
use super::util::{try_from_vec};


const DEFAULT_SCHEMA: &'static str = "bareretro";
const DEFAULT_TABLE_BOARDS: &'static str = "boards";

const DEFAULT_HOST: &'static str = "postgres";
const DEFAULT_PORT: &'static str = "5432";
const DEFAULT_USER: &'static str = "postgres";
const DEFAULT_PASSWORD: &'static str = "";
const DEFAULT_DBNAME: &'static str = "postgres";

const FIELD_ID: &'static str = "id";
const FIELD_TITLE: &'static str = "title";
const FIELD_OWNER: &'static str = "owner";
const FIELD_CREATED_AT: &'static str = "created_at";
//const FIELD_UPDATED_AT: &'static str = "updated_at";


#[derive(Clone)]
pub struct PostgresStorage {
    time_provider: Box<dyn TimeProvider>,
    schema: String,
    table_boards: String,
    pool: Pool,
}

impl TryFrom<Row> for Board {
    type Error = MyError;

    fn try_from(row: Row) -> Result<Self, Self::Error> {
        // consider: https://docs.rs/tokio-pg-mapper/0.1.8/tokio_pg_mapper/
        // https://docs.rs/tokio-postgres/0.5.5/tokio_postgres/row/struct.Row.html#method.try_get
        let id = row.try_get(&FIELD_ID).map_err(|why| format!("Could not get id! {}", why))?;
        let title = row.try_get(&FIELD_TITLE).map_err(|why| format!("Could not get title! {}", why))?;
        let owner = row.try_get(&FIELD_OWNER).map_err(|why| format!("Could not get owner! {}", why))?;
        let created_at = row.try_get(&FIELD_CREATED_AT).map_err(|why| format!("Could not get created_at! {}", why))?;

        Ok(Self {
            id: id,
            title: title,
            owner: owner,
            created_at: created_at,
        })
    }
}

impl PostgresStorage {
    pub fn from_env (time_provider: Box<dyn TimeProvider>) -> Result<Self, MyError> {
        // https://crates.io/crates/deadpool-postgres
        let cfg = DeadpoolConfig {
            host: Some(Config::env_var_string("PG_HOST", String::from(DEFAULT_HOST))),
            port: Some(
                Config::env_var_string("PG_PORT", String::from(DEFAULT_PORT))
                    .parse::<u16>().map_err(|why| format!("Port is not a valid number! {}", why))?
            ),
            user: Some(Config::env_var_string("PG_USER", String::from(DEFAULT_USER))),
            password: Some(Config::env_var_string("PG_PASS", String::from(DEFAULT_PASSWORD))),
            dbname: Some(Config::env_var_string("PG_DBNAME", String::from(DEFAULT_DBNAME))),
            ..Default::default()
        };

        let storage = Self {
            time_provider: time_provider,
            schema: Config::env_var_string("PG_SCHEMA", String::from(DEFAULT_SCHEMA)),
            table_boards: Config::env_var_string("PG_TABLE_BOARDS", String::from(DEFAULT_TABLE_BOARDS)),
            pool: cfg.create_pool(NoTls).map_err(|why| format!("Failed creating pool: {}", why))?,
        };

        Ok(storage)
    }

    async fn client (&self) -> Result<Client, MyError> {
        self.pool.get().await.map_err(|why| format!("Failed creating client: {}", why))
    }
}

// https://github.com/dtolnay/async-trait#non-threadsafe-futures
#[async_trait(?Send)]
impl Storage for PostgresStorage {
    fn name(&self) -> &'static str {
        "Postgres"
    }

    async fn add_board (&self, item: &Board) -> Result<bool, MyError> {
        match self.client().await?.execute(
            format!(
                "INSERT INTO {}.{} ({}, {}, {}, {}) VALUES ($1, $2, $3, $4)",
                self.schema,
                self.table_boards,
                FIELD_ID,
                FIELD_TITLE,
                FIELD_OWNER,
                FIELD_CREATED_AT,
            ).as_str(),
            &[
                &item.id,
                &item.title,
                &item.owner,
                &item.created_at,
            ],
        ).await {
            Err(why) => Err(format!("Add board failed: {}", why.to_string())),
            Ok(_) => Ok(true)
        }
    }

    async fn list_boards (&self) -> Result<Vec<Board>, MyError>  {
        match self.client().await?.query(
            format!(
                "SELECT {}, {}, {}, {} FROM {}.{}",
                FIELD_ID,
                FIELD_TITLE,
                FIELD_OWNER,
                FIELD_CREATED_AT,
                self.schema,
                self.table_boards,
            ).as_str(),
            &[
            ],
        ).await {
            Err(why) => Err(format!("List boards failed: {}", why.to_string())),
            Ok(rows) => try_from_vec(rows, "boards"),
        }
    }

    async fn get_board (&self, id: &String) -> Result<Board, MyError> {
        match self.client().await?.query_one(
            format!(
                "SELECT * FROM {}.{} WHERE {} = $1",
                self.schema,
                self.table_boards,
                FIELD_ID,
            ).as_str(),
            &[
                id,
            ],
        ).await {
            Err(why) => Err(format!("List boards failed: {}", why.to_string())),
            Ok(row) => match Board::try_from(row) {
                Err(why) => Err(format!("Failed converting board: {}", why)),
                Ok(item) => Ok(item),
            }
        }
    }

    async fn delete_board(&self, id: &String) -> Result<bool, MyError> {
        match self.client().await?.execute(
            format!(
                "DELETE FROM {}.{} WHERE {} = $1",
                self.schema,
                self.table_boards,
                FIELD_ID,
            ).as_str(),
            &[
                id,
            ],
        ).await {
            Err(why) => Err(format!("Delete board failed: {}", why.to_string())),
            Ok(update_count) => Ok(update_count == 0)
        }
    }
}
