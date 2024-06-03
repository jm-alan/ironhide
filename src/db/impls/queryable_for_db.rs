use crate::db::{queryable::with_client, QueryError, Queryable, DB};
use tokio_postgres::{tls::MakeTlsConnect, types::ToSql, Row, Socket, ToStatement};

impl<T> Queryable for DB<T>
where
    T: MakeTlsConnect<Socket> + Copy,
    T::Stream: Send + 'static,
{
    async fn query<U>(
        &self,
        statement: U,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<Row>, QueryError>
    where
        U: ToStatement,
    {
        if !self.connected {
            return Err(QueryError::new(
                "Database is not connected! Did you call connect()?",
            ));
        }

        // Acquire semaphore permit to ensure there's at least 1 client available in the pool
        let _permit = self.pool_semaphore.acquire().await.unwrap();

        // Lock, pop, and drop it - we only need to maintain a lock on the pool until we have a
        // client; we want to make sure we relinquish the lock before we hit the query `await` to
        // ensure that, when this function is called repeatedly in a loop, other queries can still
        // retrieve clients from the pool while this particular client.query is running
        let mut pop_pool = self.pool.lock().unwrap();
        let client_idx = pop_pool.pop_front().unwrap();
        drop(pop_pool);

        let (client_mutex, _) = self.pool_inner.get(client_idx as usize).unwrap();
        let mut client_guard = client_mutex.lock().unwrap();

        let result = with_client(&mut client_guard, statement, params).await;
        drop(client_guard);

        // Re-lock the pool and return this client to it
        let mut push_pool = self.pool.lock().unwrap();
        push_pool.push_back(client_idx);

        match result {
            Err(query_error) => Err(QueryError::new(&format!("{:?}", query_error))),
            Ok(row) => Ok(row),
        }

        // The push pool and semaphore permit will be dropped automatically
    }
}
