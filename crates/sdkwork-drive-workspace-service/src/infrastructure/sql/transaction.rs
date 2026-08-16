/// Returns the PostgreSQL statement that starts a write transaction.
pub fn begin_transaction_sql() -> &'static str {
    "BEGIN"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn begins_postgres_transaction() {
        assert_eq!(begin_transaction_sql(), "BEGIN");
    }
}
