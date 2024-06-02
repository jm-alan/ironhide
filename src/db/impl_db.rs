use std::fmt::Debug;

use tokio_postgres::{tls::MakeTlsConnect, types::ToSql, Error, Row, Socket, ToStatement};

use super::{db_builder::DBBuilder, DB};

impl<T> DB<T>
where
    T: MakeTlsConnect<Socket> + Copy,
    T::Stream: Send + 'static,
{
    pub fn builder() -> DBBuilder<T> {
        DBBuilder::new()
    }

    pub async fn connect(&mut self) {
        let mut push_pool = self.pool.lock().unwrap();
        for _ in 0..self.pool_size {
            let formatted = format!(
                "host={} port={} dbname={} user={} password={}",
                self.host, self.port, self.name, self.role, self.password
            );

            match tokio_postgres::connect(&formatted, self.tls).await {
                Ok((client, connection)) => {
                    let handle = tokio::spawn(async move {
                        if let Err(e) = connection.await {
                            panic!("connection error: {}", e);
                        }
                    });

                    match client.query("SELECT 1", &[]).await {
                        Ok(_) => {}
                        Err(err) => panic!("Connection test failed {:?}", err),
                    };

                    push_pool.push_back((client, handle));
                }
                Err(err) => panic!("Boom! Fire! Explosions! Sad faces! :( {:?}", err),
            }
        }

        self.connected = true;
    }

    pub async fn query<U>(
        &self,
        statement: U,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<Row>, Error>
    where
        U: ToStatement + Debug,
    {
        if !self.connected {
            panic!("Database is not connected! Did you call connect()?");
        }

        // Acquire semaphore permit to ensure there's at least 1 client available in the pool
        let _permit = self.pool_semaphore.acquire().await.unwrap();

        // Lock, pop, and drop it - we only need to maintain a lock on the pool until we have a
        // client; we want to make sure we relinquish the lock before we hit the query `await` to
        // ensure that, when this function is called repeatedly in a loop, other queries can still
        // retrieve clients from the pool simultaneously
        let mut pop_pool = self.pool.lock().unwrap();
        let (client, handle) = pop_pool.pop_front().unwrap();
        drop(pop_pool);

        let result = client.query(&statement, params).await;

        // Re-lock the pool and return this client to it
        let mut push_pool = self.pool.lock().unwrap();
        push_pool.push_back((client, handle));

        return result;

        // The push pool and semaphore permit will be dropped automatically
    }
}

// If counter == pool_size
