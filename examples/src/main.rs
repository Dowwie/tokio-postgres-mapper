#[macro_use]
extern crate tokio_pg_mapper_derive;

#[derive(PostgresMapper)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
}

fn main() {}
