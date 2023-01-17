pub mod models;
pub mod schema;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

use self::models::*;
use self::schema::keys::dsl::{address, keys, local_share};
use diesel::result::Error::{DatabaseError, NotFound};
use diesel::result::DatabaseErrorKind::UniqueViolation;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn get_local_share(conn: &mut PgConnection, query_address: &str) -> QueryResult<String> {
    let result = keys.filter(address.eq(query_address)).load::<Key>(conn)?;

    if result.len() == 0 {
        return Err(NotFound);
    } else if result.len() != 1 {
        return Err(DatabaseError(
            UniqueViolation,
            Box::new("found multiple entries".to_string()),
        ));
    }

    Ok(result[0].local_share.to_owned())
}

pub fn insert_new_key(conn: &mut PgConnection) -> i32 {
    use crate::db::schema::keys;

    let key_inserted: Key = diesel::insert_into(keys::table)
        .values((address.eq(""), local_share.eq("")))
        .get_result(conn)
        .expect("Error saving new key");

    key_inserted.id
}

pub fn fill_in_key_data(
    conn: &mut PgConnection,
    id: i32,
    adress_data: &str,
    local_share_data: &str,
) {
    diesel::update(keys.find(id))
        .set((address.eq(adress_data), local_share.eq(local_share_data)))
        .get_result::<Key>(conn)
        .unwrap();
}
