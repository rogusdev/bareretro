
use async_trait::async_trait;

use crate::models::{MyError, Storage, Board};


#[derive(Clone)]
pub struct InvalidStorage {
    pub error: String,
}

// https://github.com/dtolnay/async-trait#non-threadsafe-futures
#[async_trait(?Send)]
impl Storage for InvalidStorage {
    fn name(&self) -> &'static str {
        "INVALID"
    }

    async fn add_board (&self, _item: &Board) -> Result<bool, MyError> {
        Err(self.error.clone())
    }

    async fn list_boards (&self) -> Result<Vec<Board>, MyError>  {
        Err(self.error.clone())
    }

    async fn delete_board (&self, _id: &String) -> Result<bool, MyError> {
        Err(self.error.clone())
    }
}
