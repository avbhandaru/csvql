use crate::table;
use async_trait::async_trait;
use tokio_postgres::error::Error;

#[async_trait]
pub trait QuerierTrait {
  async fn store(
    &self,
    table_name: &str,
    table_header: table::Header,
    table_data: table::Rows,
  ) -> Result<(), Error>;
  async fn load(&self, table_name: &str) -> Result<table::Table, Error>;
  async fn query(&self, query_statement: &str) -> Result<table::Table, Error>;
}
