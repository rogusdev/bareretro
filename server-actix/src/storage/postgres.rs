
use std::convert::TryFrom;
use async_trait::async_trait;

use deadpool_postgres::{Client, Config as DeadpoolConfig, Pool};
use tokio_postgres::{NoTls, row::Row, types::{FromSql, ToSql}};

use crate::time_provider::TimeProvider;
use crate::models::{MyError, Config, Storage, Board, Column};
use super::util::{try_from_vec};
use const_format::formatcp;


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
// const FIELD_CONTENTS: &'static str = "contents";
// const FIELD_AUTHOR: &'static str = "author";
const FIELD_CREATED_AT: &'static str = "created_at";
//const FIELD_UPDATED_AT: &'static str = "updated_at";

const FIELD_BOARD_ID: &'static str = "board_id";
// const FIELD_COLUMN_ID: &'static str = "column_id";
// const FIELD_CARD_ID: &'static str = "card_id";


#[derive(Clone)]
pub struct PostgresStorage {
    time_provider: Box<dyn TimeProvider>,
    schema: String,
    table_boards: String,
    pool: Pool,
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

trait RowCrud {
    fn name_single () -> &'static str;
    fn name_plural () -> &'static str;
    fn table_name (storage: &PostgresStorage) -> &String;
//    fn field_names () -> Vec<&'static str>;
    fn field_names () -> &'static str;
    // https://docs.rs/tokio-postgres/0.7.0/tokio_postgres/struct.Client.html#method.execute
    fn row_values (&self) -> Vec<&(dyn ToSql + Sync)>;
}

fn get_field<'a, T> (row: &'a Row, field: &'static str) -> Result<T, MyError> where T: FromSql<'a> {
    row.try_get(field).map_err(|why| format!("Could not get {}! {}", field, why))
}

fn values_str<T> (values: &Vec<T>) -> String {
    // TODO: have prepared consts for all anticipated lengths? or at least cache for sizes?
    (1..values.len())
        .map(|n| format!("${}", n))
        .collect::<Vec<String>>()
        .join(", ")
}

async fn add<T> (storage: &PostgresStorage, item: &T) -> Result<bool, MyError> where T: RowCrud {
    let values = item.row_values();
    match storage.client().await?.execute(
        format!(
            "INSERT INTO {}.{} ({}) VALUES ({})",
            storage.schema,
            T::table_name(&storage),
            T::field_names(),
            values_str(&values),
        ).as_str(),
        &values,
    ).await {
        Err(why) => Err(format!("Add {} failed: {}", T::name_single(), why.to_string())),
        Ok(_) => Ok(true)
    }
}

async fn list<T> (storage: &PostgresStorage) -> Result<Vec<T>, MyError>
        where T: RowCrud + TryFrom<Row, Error=MyError> {
    match storage.client().await?.query(
        format!(
            "SELECT {} FROM {}.{}",
            T::field_names(),
            storage.schema,
            T::table_name(&storage),
        ).as_str(),
        &[
        ],
    ).await {
        Err(why) => Err(format!("List {} failed: {}", T::name_plural(), why.to_string())),
        Ok(rows) => try_from_vec(rows, T::name_plural()),
    }
}

async fn get<T> (storage: &PostgresStorage, id: &String) -> Result<T, MyError>
        where T: RowCrud + TryFrom<Row, Error=MyError> {
    match storage.client().await?.query_one(
        format!(
            "SELECT * FROM {}.{} WHERE {} = $1",
            storage.schema,
            T::table_name(&storage),
            FIELD_ID,
        ).as_str(),
        &[
            id,
        ],
    ).await {
        Err(why) => Err(format!("List {} failed: {}", T::name_plural(), why.to_string())),
        Ok(row) => match T::try_from(row) {
            Err(why) => Err(format!("Failed converting {}: {}", T::name_single(), why)),
            Ok(item) => Ok(item),
        }
    }
}

async fn delete<T> (storage: &PostgresStorage, id: &String) -> Result<bool, MyError> where T: RowCrud {
    match storage.client().await?.execute(
        format!(
            "DELETE FROM {}.{} WHERE {} = $1",
            storage.schema,
            T::table_name(&storage),
            FIELD_ID,
        ).as_str(),
        &[
            id,
        ],
    ).await {
        Err(why) => Err(format!("Delete {} failed: {}", T::name_single(), why.to_string())),
        Ok(update_count) => Ok(update_count == 0)
    }
}


// https://github.com/dtolnay/async-trait#non-threadsafe-futures
#[async_trait(?Send)]
impl Storage for PostgresStorage {
    fn name (&self) -> &'static str {
        "Postgres"
    }

    // async fn add<T> (&self, item: &T) -> Result<bool, MyError> {
    //     add(self, item).await
    // }

    // async fn list<T> (&self) -> Result<Vec<T>, MyError>  {
    //     list(self).await
    // }

    // async fn get<T> (&self, id: &String) -> Result<T, MyError> {
    //     get(self, id).await
    // }

    // async fn delete<T> (&self, id: &String) -> Result<bool, MyError> {
    //     delete::<T>(self, id).await
    // }

    // BOARDS
    async fn add_board (&self, item: &Board) -> Result<bool, MyError> {
        add(self, item).await
    }

    async fn list_boards (&self) -> Result<Vec<Board>, MyError>  {
        list(self).await
    }

    async fn get_board (&self, id: &String) -> Result<Board, MyError> {
        get(self, id).await
    }

    async fn delete_board (&self, id: &String) -> Result<bool, MyError> {
        delete::<Board>(self, id).await
    }

    // COLUMNS
    async fn add_column (&self, item: &Column) -> Result<bool, MyError> {
        add(self, item).await
    }

    async fn list_columns (&self) -> Result<Vec<Column>, MyError>  {
        list(self).await
    }

    async fn get_column (&self, id: &String) -> Result<Column, MyError> {
        get(self, id).await
    }

    async fn delete_column (&self, id: &String) -> Result<bool, MyError> {
        delete::<Column>(self, id).await
    }
}


const BOARD_SINGLE: &'static str = "Board";
const BOARD_PLURAL: &'static str = "Boards";
const BOARD_FIELDS: &'static str = formatcp!(
    "{}, {}, {}, {}",
    FIELD_ID,
    FIELD_TITLE,
    FIELD_OWNER,
    FIELD_CREATED_AT,
);

impl RowCrud for Board {
    fn name_single () -> &'static str {
        BOARD_SINGLE
    }

    fn name_plural () -> &'static str {
        BOARD_PLURAL
    }

    fn table_name (storage: &PostgresStorage) -> &String {
        &storage.table_boards
    }

    fn field_names () -> &'static str {
        BOARD_FIELDS
    }

    fn row_values (&self) -> Vec<&(dyn ToSql + Sync)> {
        vec![
            &self.id,
            &self.title,
            &self.owner,
            &self.created_at,
        ]
    }
}

impl TryFrom<Row> for Board {
    type Error = MyError;

    fn try_from (row: Row) -> Result<Self, Self::Error> {
        // consider: https://docs.rs/tokio-pg-mapper/0.1.8/tokio_pg_mapper/
        // https://docs.rs/tokio-postgres/0.5.5/tokio_postgres/row/struct.Row.html#method.try_get
        Ok(Self {
            id: get_field(&row, FIELD_ID)?,
            title: get_field(&row, FIELD_TITLE)?,
            owner: get_field(&row, FIELD_OWNER)?,
            created_at: get_field(&row, FIELD_CREATED_AT)?,
        })
    }
}



const COLUMN_SINGLE: &'static str = "Column";
const COLUMN_PLURAL: &'static str = "Columns";
const COLUMN_FIELDS: &'static str = formatcp!(
    "{}, {}, {}, {}",
    FIELD_ID,
    FIELD_BOARD_ID,
    FIELD_TITLE,
    FIELD_CREATED_AT,
);

impl RowCrud for Column {
    fn name_single () -> &'static str {
        COLUMN_SINGLE
    }

    fn name_plural () -> &'static str {
        COLUMN_PLURAL
    }

    fn table_name (storage: &PostgresStorage) -> &String {
        &storage.table_boards
    }

    fn field_names () -> &'static str {
        COLUMN_FIELDS
    }

    fn row_values (&self) -> Vec<&(dyn ToSql + Sync)> {
        vec![
            &self.id,
            &self.board_id,
            &self.title,
            &self.created_at,
        ]
    }
}

impl TryFrom<Row> for Column {
    type Error = MyError;

    fn try_from (row: Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: get_field(&row, FIELD_ID)?,
            board_id: get_field(&row, FIELD_BOARD_ID)?,
            title: get_field(&row, FIELD_TITLE)?,
            created_at: get_field(&row, FIELD_CREATED_AT)?,
        })
    }
}
