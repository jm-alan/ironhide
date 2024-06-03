mod db;

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use db::Queryable;
    use db::DB;
    use futures::future::join_all;
    use tokio_postgres::NoTls;

    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_connect() {
        let row_count = 10000;

        for pool_size in [1, 5, 10, 20] {
            let mut pooled_db = DB::builder()
                .host("localhost")
                .port(5432)
                .name("ironhide_proving_ground")
                .role("ironhide_app_generic")
                .password("P@ssw0rd")
                .tls(NoTls)
                .pool_size(pool_size)
                .build();

            pooled_db.connect().await;

            let mut insert_futures = vec![];

            let then = Instant::now();

            for i in 0..row_count {
                insert_futures.push(pooled_db.query(
                    format!("INSERT INTO landfill (some_val) VALUES ({i});"),
                    &[],
                ));
            }

            join_all(insert_futures).await;

            let elapsed = then.elapsed();

            println!(
                "Inserting {row_count} rows with {pool_size} connection(s) took {:?}",
                elapsed
            );

            let Ok(results) = pooled_db
                .query("SELECT * FROM landfill;".to_owned(), &[])
                .await
            else {
                panic!("Failed to retrieve results after insert");
            };

            println!("Select confirmed inserted rows: {}", results.len());
            let Ok(_) = pooled_db
                .query("DELETE FROM landfill;".to_owned(), &[])
                .await
            else {
                panic!("Failed to run cleanup after tests");
            };
        }
    }
}
