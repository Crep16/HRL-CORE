mod connection;
pub mod migrations;

pub use connection::Database;

#[cfg(test)]
mod tests;
