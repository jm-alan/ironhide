use tokio_postgres::NoTls;

#[test]
pub fn test_schema_derive() {
    use crate::db::DB;
    use ironhide_macros::{schema, Schema};

    #[derive(Schema)]
    #[schema(name = "user")]
    struct User {
        id: String,
        age: i64,
    }

    let mut db = DB::builder()
        .host("localhost")
        .port(5432)
        .name("ironhide_proving_ground")
        .role("ironhide_app_generic")
        .password("P@ssw0rd")
        .tls(NoTls)
        .pool_size(10)
        .build();

    db.connect();

    let user: Vec<User> = db.all();
}
