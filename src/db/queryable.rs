use tokio_postgres::{types::ToSql, Client, Row, ToStatement};

use super::QueryError;

pub trait Queryable {
    async fn query<T>(
        &self,
        statement: T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<Row>, QueryError>
    where
        T: ToStatement;
}

#[inline(always)]
pub(crate) async fn with_client<T>(
    client: &mut Client,
    statement: T,
    params: &[&(dyn ToSql + Sync)],
) -> Result<Vec<Row>, tokio_postgres::Error>
where
    T: ToStatement,
{
    client.query(&statement, params).await
}
