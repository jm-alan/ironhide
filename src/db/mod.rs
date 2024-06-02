mod db_builder;
mod impl_db;

use std::{collections::VecDeque, sync::Mutex};
use tokio::{sync::Semaphore, task::JoinHandle};
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
    pool_size: usize,
    pool: Mutex<VecDeque<(Client, JoinHandle<()>)>>,
    pool_semaphore: Semaphore,
    pub connected: bool,
}
