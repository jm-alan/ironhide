use std::{collections::VecDeque, sync::Mutex};

use tokio::sync::Semaphore;
use tokio_postgres::{tls::MakeTlsConnect, Socket};

use super::DB;

pub(crate) struct DBBuilder<T>
where
    T: MakeTlsConnect<Socket> + Copy,
    T::Stream: Send + 'static,
{
    host: Option<String>,
    port: Option<u16>,
    name: Option<String>,
    role: Option<String>,
    password: Option<String>,
    pool_size: Option<usize>,
    tls: Option<T>,
}

impl<T> DBBuilder<T>
where
    T: MakeTlsConnect<Socket> + Copy,
    T::Stream: Send + 'static,
{
    pub fn new() -> Self {
        Self {
            host: None,
            port: None,
            name: None,
            role: None,
            password: None,
            pool_size: None,
            tls: None,
        }
    }

    pub fn role(mut self, role: &str) -> Self {
        self.role = Some(role.to_owned());

        self
    }

    pub fn password(mut self, password: &str) -> Self {
        self.password = Some(password.to_owned());

        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());

        self
    }

    pub fn host(mut self, host: &str) -> Self {
        self.host = Some(host.to_owned());

        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);

        self
    }

    pub fn pool_size(mut self, pool_size: usize) -> Self {
        self.pool_size = Some(pool_size);

        self
    }

    pub fn tls(mut self, tls: T) -> Self {
        self.tls = Some(tls);

        self
    }

    pub fn build(self) -> DB<T> {
        let Some(role) = self.role else {
            panic!("DB connection role not provided");
        };

        let Some(name) = self.name else {
            panic!("DB name not provided");
        };

        let Some(tls) = self.tls else {
            panic!("TLS config not provided");
        };

        let password = self.password.unwrap_or_default();
        let host = self.host.unwrap_or("localhost".to_owned());
        let port = self.port.unwrap_or(5432);
        let pool_size = self.pool_size.unwrap_or(1);

        DB {
            host,
            port,
            name,
            role,
            password,
            tls,
            pool_size,
            pool: Mutex::new(VecDeque::new()),
            pool_semaphore: Semaphore::new(pool_size),
            connected: false,
        }
    }
}
