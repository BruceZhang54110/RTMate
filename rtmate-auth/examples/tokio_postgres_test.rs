use tokio_postgres::{Error, GenericClient, NoTls, SimpleQueryMessage, SimpleQueryRow};

#[tokio::main] // By default, tokio_postgres uses the tokio crate as its runtime.
async fn main() -> Result<(), Error> {
    // Connect to the database.
    let (client, connection) =
        tokio_postgres::connect("hostaddr=127.0.0.1 host=localhost port=5432 user=rtmate_role dbname=rtmate password=123456", NoTls)
        .await?;

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    println!("Connected to the database successfully!");

    // Now we can execute a simple statement that just returns its parameter.
    let rows = client
        .query("SELECT $1::TEXT", &[&"hello world"])
        .await?;

    // And then check that we got back the same string we sent over.
    let value: &str = rows[0].get(0);
    assert_eq!(value, "hello world");
    println!("finish, value: {}", value);


    let row = client.simple_query("select * from rt_app limit 1").await?;
    //println!("Row: {:?}", row);
    let rows: Vec<SimpleQueryRow> = row.into_iter().filter_map(|row| {
        if let SimpleQueryMessage::Row(row) = row {
            Some(row)
        } else {
            None
        }
    }).collect();
    rows.get(0).map(|r| {
        println!("Row data: {:?}", r.get("app_id"));
    });

    Ok(())
}