//! # tokio-pg-mapper
//!
//! `tokio-pg-mapper` is a proc-macro designed to make mapping from postgresql
//! tables to structs simple.
//!
//! ### Why?
//!
//! It can be frustrating to write a lot of boilerplate and, ultimately, duplicated
//! code for mapping from postgres Rows into structs.
//!
//! For example, this might be what someone would normally write:
//!
//! ```rust
//! use postgres::row::Row;
//!
//! pub struct User {
//!     pub id: i64,
//!     pub name: String,
//!     pub email: Option<String>,
//! }
//!
//! impl From<Row> for User {
//!     fn from(row: Row) -> Self {
//!         Self {
//!             id: row.get("id"),
//!             name: row.get("name"),
//!             email: row.get("email"),
//!         }
//!     }
//! }
//!
//! // code to execute a query here and get back a row
//! let user = User::from_row(row); // returns Result<User, tokio_pg_mapper::Error>
//! ```
//!
//!
//! ### The two crates
//!
//! This repository contains two crates: `tokio-pg-mapper` which contains an `Error`
//! enum and traits for converting from `tokio-postgres` `Row`
//! without panicking, and `pg-mapper-derive` which contains the proc-macro.
//!
//! `pg-mapper-derive` has 3 features that can be enabled (where T is the
//! struct being derived with the provided `TokioPostgresMapper` proc-macro):
//!
//! `impl FromTokioPostgresRow<::tokio_postgres::row::Row> for T` and
//! `impl FromTokioPostgresRow<&::tokio_postgres::row::Row> for T` implementations
//! - `pg-mapper` which, for each of the above features, implements
//! `pg-mapper`'s `FromTokioPostgresRow` trait
//!
//!
//! This will derive implementations for converting from owned and referenced
//! `tokio-postgres::row::Row`s, as well as implementing `pg-mapper`'s
//! `FromTokioPostgresRow` trait for non-panicking conversions.
#[cfg(feature = "derive")]
#[allow(unused_imports)]
pub extern crate tokio_pg_mapper_derive;

#[cfg(feature = "derive")]
#[doc(hidden)]
pub use tokio_pg_mapper_derive::*;

use tokio_postgres;
use tokio_postgres::row::Row as TokioRow;

use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Trait containing various methods for converting from a `tokio-postgres` Row
/// to a mapped type.
///
/// When using the `pg_mapper_derive` crate's `TokioPostgresMapper` proc-macro,
/// this will automatically be implemented on types.
///
/// The [`from_row`] method exists for consuming a `Row` - useful
/// for iterator mapping - while [`from_row_ref`] exists for borrowing
/// a `Row`.
pub trait FromTokioPostgresRow: Sized {
    /// Converts from a `tokio-postgres` `Row` into a mapped type, consuming the
    /// given `Row`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ColumnNotFound`] if the column in a mapping was not
    /// found.
    ///
    /// Returns [`Error::Conversion`] if there was an error converting the row
    /// column to the requested type.
    ///
    /// [`Error::ColumnNotFound`]: enum.Error.html#variant.ColumnNotFound
    /// [`Error::Conversion`]: enum.Error.html#variant.Conversion
    fn from_row(row: TokioRow) -> Result<Self, Error>;

    /// Converts from a `tokio-postgres` `Row` into a mapped type, borrowing the
    /// given `Row`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ColumnNotFound`] if the column in a mapping was not
    /// found.
    ///
    /// Returns [`Error::Conversion`] if there was an error converting the row
    /// column into the requested type.
    ///
    /// [`Error::ColumnNotFound`]: enum.Error.html#variant.ColumnNotFound
    /// [`Error::Conversion`]: enum.Error.html#variant.Conversion
    fn from_row_ref(row: &TokioRow) -> Result<Self, Error>;

    /// Get the name of the annotated sql table name.
    ///
    /// Example:
    ///
    /// The following will return the String " user ".
    /// Note the extra spaces on either side to avoid incorrect formatting.
    ///
    /// ```
    ///     #[derive(PostgresMapper)]
    ///     #[pg_mapper(table = "user")]
    ///     pub struct User {
    ///         pub id: i64,
    ///         pub email: Option<String>,
    ///     }
    /// ```
    fn sql_table() -> String;
    
    
    /// Get a list of the field names, excluding table name prefix. 
    ///
    /// Example:
    ///
    /// The following will return the String " id, email ".
    /// Note the extra spaces on either side to avoid incorrect formatting.
    ///
    /// ```
    ///     #[derive(PostgresMapper)]
    ///     #[pg_mapper(table = "user")]
    ///     pub struct User {
    ///         pub id: i64,
    ///         pub email: Option<String>,
    ///     }
    /// ```
    ///
    fn sql_fields() -> String;

    /// Get a list of the field names, including table name prefix.
    ///
    /// We also expect an attribute tag #[pg_mapper(table = "foo")]
    /// so that a scoped list of fields can be generated.
    ///
    /// Example:
    ///
    /// The following will return the String " user.id, user.email ".
    /// Note the extra spaces on either side to avoid incorrect formatting.
    ///
    /// ```
    ///     #[derive(PostgresMapper)]
    ///     #[pg_mapper(table = "user")]
    ///     pub struct User {
    ///         pub id: i64,
    ///         pub email: Option<String>,
    ///     }
    /// ```
    ///
    fn sql_table_fields() -> String;
}

/// General error type returned throughout the library.
#[derive(Debug)]
pub enum Error {
    /// A column in a row was not found.
    ColumnNotFound,
    /// An error from the `tokio-postgres` crate while converting a type.
    Conversion(Box<dyn StdError + Send + Sync>),
    /// Used in a scenario where tokios_postgres::Error::into_source returns None
    UnknownTokioPG
}

impl From<tokio_postgres::Error> for Error {
    fn from(err: tokio_postgres::Error) -> Self {
        if let Some(source) = err.into_source() {
            source.into()
        } else {
            Error::UnknownTokioPG
        }
    }
}

impl From<Box<dyn StdError + Send + Sync>> for Error {
    fn from(err: Box<dyn StdError + Send + Sync>) -> Self {
        Error::Conversion(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ColumnNotFound => "Column in row not found",
            Error::Conversion(ref inner) => inner.description(),
            Error::UnknownTokioPG => "Unknown/unsourced tokio-postgres error"
        }
    }
}
