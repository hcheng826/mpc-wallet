pub mod models;
pub mod schema;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

use self::models::*;
use self::schema::share2_keys::dsl::{share2_keys, id, address, local_share};
use diesel::result::Error::{DatabaseError, NotFound};
use diesel::result::DatabaseErrorKind::UniqueViolation;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn get_local_share(conn: &mut PgConnection, query_address: &str) -> QueryResult<String> {
    let result = share2_keys.filter(address.eq(query_address)).load::<Key>(conn)?;

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

pub fn insert_new_key(conn: &mut PgConnection, id_data: i32, adress_data: &str, local_share_data: &str) -> QueryResult<Key> {
    use crate::db::schema::share2_keys;

    diesel::insert_into(share2_keys::table)
        .values((id.eq(id_data), address.eq(adress_data), local_share.eq(local_share_data)))
        .get_result(conn)
}
