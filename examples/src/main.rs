use tokio_pg_mapper::PostgresMapper;

#[derive(PostgresMapper)]
#[pg_mapper(table = "user")]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
}

fn main() {}
