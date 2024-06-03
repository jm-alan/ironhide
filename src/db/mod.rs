mod db_builder;
mod impls;
mod query_error;
mod queryable;

pub use query_error::QueryError;
pub use queryable::Queryable;
use std::{collections::VecDeque, sync::Mutex};
use tokio::{
    runtime::Handle,
    sync::Semaphore,
    task::{block_in_place, JoinHandle},
};
use tokio_postgres::{tls::MakeTlsConnect, Client, Socket};

pub struct DB<T>
where
    T: MakeTlsConnect<Socket> + Copy,
    T::Stream: Send + 'static,
{
    host: String,
    port: u16,
    name: String,
    role: String,
    password: String,
    tls: T,
    pool_size: u32,
    pool_inner: Vec<(Mutex<Client>, JoinHandle<()>)>,
    pool: Mutex<VecDeque<u32>>,
    pool_semaphore: Semaphore,
    pub connected: bool,
}

impl<T> Drop for DB<T>
where
    T: MakeTlsConnect<Socket> + Copy,
    T::Stream: Send + 'static,
{
    fn drop(&mut self) {
        block_in_place(|| {
            if let Err(err) =
                Handle::current().block_on(self.pool_semaphore.acquire_many(self.pool_size))
            {
                eprintln!("Failed to acquire lock on all DB semaphore permits during drop. You may have dangling connections");
                eprintln!("{:?}", err);
            };

            println!(
                "Successfully acquired full semaphore suite; connections should drop safely..."
            );
        })
    }
}
