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
#[macro_use] extern crate tokio_pg_mapper_derive;
extern crate tokio_pg_mapper;

use tokio_pg_mapper::FromPostgresRow;

#[derive(PostgresMapper)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
}

// Code to execute a query here and get back a row might now look like:
let stmt = "SELECT * FROM user WHERE username = $1 AND password = $2";

let result = client.query_one(stmt, &[&5, "asdf"]).await?;
let user = User::from(result);

```


### The two crates

This repository contains two crates: `postgres-mapper` which contains an `Error`
enum and traits for converting from a `postgres` or `tokio-pg` `Row`
without panicking, and `postgres-mapper-derive` which contains the proc-macro.

`postgres-mapper-derive` has 3 features that can be enabled (where T is the
struct being derived with the provided `PostgresMapper` proc-macro):

- `postgres-support`, which derives
`impl<'a> From<::postgres::rows::Row<'a>> for T` and
`impl<'a> From<&'a ::postgres::Row<'a>> for T` implementations
- `tokio-pg-support`, which derives
`impl From<::tokio_postgres::rows::Row> for T` and
`impl From<&::tokio_postgres::rows::Row> for T` implementations
- `tokio-pg-mapper` which, for each of the above features, implements
`tokio-pg-mapper`'s `FromPostgresRow` and/or `FromTokioPostgresRow` traits

`tokio-pg-mapper` has two features, `postgres-support` and
`tokio-pg-support`. When one is enabled in `tokio-pg-mapper-derive`, it
must also be enabled in `tokio-pg-mapper`.

### Installation

Add the following to your `Cargo.toml`:

```toml
tokio-pg-mapper = { git = "https://github.com/Dowwie/tokio-pg-mapper" }
tokio-pg-mapper-derive = { git = "https://github.com/Dowwie/tokio-pg-mapper" }
```

This will derive implementations for converting from owned and referenced
`tokio-pg::rows::Row`s, as well as implementing `tokio-pg-mapper`'s
`FromTokioPostgresRow` trait for non-panicking conversions.

### License

ISC.
