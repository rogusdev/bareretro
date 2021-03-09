
use std::env;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use dyn_clonable::clonable;

use crate::time_provider::TimeProvider;


const EMPTY_STRING: String = String::new();


pub type MyError = String;

#[derive(Debug, Clone)]
pub struct Config {
    pub provider: String,
}

impl Config {
    pub fn env_var_string (name: &str, default: String) -> String {
        env::var(name).unwrap_or(default)
    }

    #[allow(unused)]
    fn env_var_parse<T : std::str::FromStr> (name: &str, default: T) -> T {
        match env::var(name) {
            Ok(s) => s.parse::<T>().unwrap_or(default),
            _ => default
        }
    }

    // maybe TODO? https://github.com/actix/examples/blob/ec6e14aacc10bf4d44309ddb73fe01f9c27faf6f/async_pg/src/main.rs#L10
    // seems very ubiquitous: https://crates.io/crates/config
    pub fn from_env () -> Config {
        Config {
            provider: Self::env_var_string("STORAGE_PROVIDER", EMPTY_STRING),
        }
    }
}

// https://github.com/dtolnay/async-trait#non-threadsafe-futures
#[async_trait(?Send)]
#[clonable]
pub trait Storage : Clone {
    fn name(&self) -> &'static str;
    // BOARDS
    async fn add_board (&self, item: &Board) -> Result<bool, MyError>;
    async fn list_boards (&self) -> Result<Vec<Board>, MyError>;
    async fn get_board (&self, id: &String) -> Result<Board, MyError>;
    async fn delete_board (&self, id: &String) -> Result<bool, MyError>;
    // COLUMNS
    async fn add_column (&self, item: &Column) -> Result<bool, MyError>;
    async fn list_columns (&self) -> Result<Vec<Column>, MyError>;
    async fn get_column (&self, id: &String) -> Result<Column, MyError>;
    async fn delete_column (&self, id: &String) -> Result<bool, MyError>;
}

#[derive(Clone)]
pub struct Service {
    // box vs generics: dynamic vs static dispatch
    // https://stackoverflow.com/questions/48833009/the-fold-method-cannot-be-invoked-on-a-trait-object
    pub time_provider: Box<dyn TimeProvider>,
    pub config: Config,
    pub storage: Box<dyn Storage>,
}


#[derive(Debug, Clone, Serialize)]
pub struct Board {
    pub id: String,
    pub title: String,
    pub owner: String,
    pub created_at: i64,
}

#[derive(Deserialize)]
pub struct CreateBoard {
    pub title: String,
    pub owner: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Column {
    pub id: String,
    pub board_id: String,
    pub title: String,
    pub created_at: i64,
}

#[derive(Deserialize)]
pub struct CreateColumn {
    pub board_id: String,
    pub title: String,
}
