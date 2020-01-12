# tokio-pg-mapper

`tokio_postgres-mapper` is a proc-macro designed to make mapping from postgresql
tables to structs simple.

### Why?

It can be frustrating to write a lot of boilerplate and, ultimately, duplicated
code for mapping from postgres Rows into structs.

For example, this might be what someone would normally write:

```rust
extern crate postgres;

use postgres::rows::Row;

pub struct User {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
}

impl From<Row> for User {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            name: row.get("name"),
            email: row.get("email"),
        }
    }
}

// code to execute a query here and get back a row
let user = User::from(row); // this can panic
```

This becomes worse when manually implementating using the non-panicking
`get_opt` method variant.

Using this crate, the boilerplate is removed, and panicking and non-panicking
implementations are derived:

```rust
extern crate tokio_pg_mapper_derive;
extern crate tokio_pg_mapper;

use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(PostgresMapper)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
}

// Code to execute a query here and get back a row might now look like:
let stmt = "SELECT * FROM user WHERE username = $1 AND password = $2";

let result = client.query_one(stmt, &[&5, "asdf"]).await?;
let user = User::from_row(result).unwrap(); // or from_row_ref(&result)


```


### The two crates

This repository contains two crates: `postgres-mapper` which contains an `Error`
enum and traits for converting from a `tokio-postgres` `Row`
without panicking, and `postgres-mapper-derive` which contains the proc-macro.


### Installation

Install `tokio-pg-mapper-derive` and `tokio-pg-mapper` from crates.io


### License

ISC.
