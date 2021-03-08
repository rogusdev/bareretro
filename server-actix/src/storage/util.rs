
use std::convert::TryFrom;
//use std::fmt::Display;

use crate::models::MyError;


// https://users.rust-lang.org/t/impl-tryinto-as-an-argument-in-a-function-complains-about-the-error-conversion/34004
pub fn try_from_vec<T, U: TryFrom<T, Error=MyError>> (rows: Vec<T>, name: &'static str) -> Result<Vec<U>, MyError>  {
    let mut vec = Vec::new();
    for row in rows.into_iter() {
        match U::try_from(row) {
            Err(why) => return Err(format!("Failed converting {}: {}", name, why)),
            Ok(res) => vec.push(res),
        }
    }
    Ok(vec)
}
