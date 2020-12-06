use crate::table::{EntryType, Header, Rows};
use async_trait::async_trait;
use std::vec::Vec;
use tokio_postgres::{Client, Error};

#[derive(Debug)]
pub struct QueryResponse {
  pub header: Header,
  pub rows: Rows,
}

#[derive(Debug)]
pub struct Querier {
  pub name: String,
  pub url: String,
  pub client: Client,
}

#[async_trait]
pub trait QuerierTrait {
  async fn new(querier_name: &str, database_url: &str) -> Result<Querier, Error>;
  async fn store(
    &self,
    table_name: &str,
    table_header: Header,
    table_data: Rows,
  ) -> Result<(), Error>;
  async fn load(&self, table_name: &str) -> Result<QueryResponse, Error>;
  async fn query(&self, query_statement: &str) -> Result<QueryResponse, Error>;
}
