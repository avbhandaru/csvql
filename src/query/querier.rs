use crate::table;
use async_trait::async_trait;
use tokio_postgres::error::Error;

#[async_trait]
pub trait QuerierTrait {
  async fn store(
    &self,
    table_path: &str,
    table_name: &str,
    table_header: table::Header,
    // table_data: table::Rows,
  ) -> Result<(), Error>;
  async fn load(
    &self,
    table_name: &str,
    num_rows: Option<usize>,
  ) -> Result<Option<table::Table>, Error>;
  async fn query(&self, query_statement: &str) -> Result<Option<table::Table>, Error>;
  async fn drop(&self, table_name: &str) -> Result<(), Error>;
  async fn list(&self) -> Result<Option<table::Table>, Error>;
  async fn info(&self, table_name: &str, is_verbose: bool) -> Result<Option<table::Table>, Error>;
}
