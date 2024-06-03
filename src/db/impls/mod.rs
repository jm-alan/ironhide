mod queryable_for_db;

use std::sync::Mutex;

use tokio_postgres::{tls::MakeTlsConnect, Socket};

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
        if self.connected {
            return;
        }

        let mut push_pool = self.pool.lock().unwrap();

        for idx in 0..self.pool_size {
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

                    self.pool_inner.push((Mutex::new(client), handle));
                    push_pool.push_back(idx);
                }
                Err(err) => panic!("Boom! Fire! Explosions! Sad faces! :( {:?}", err),
            }
        }

        self.connected = true;
    }
}
