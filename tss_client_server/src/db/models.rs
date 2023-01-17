use diesel::prelude::*;

#[derive(Queryable, Debug)]
pub struct Key {
    pub id: i32,
    pub address: String,
    pub local_share: String,
}
