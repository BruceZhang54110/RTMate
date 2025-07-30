
use anyhow::Ok;
use tokio_postgres::tls::NoTlsStream;
use tokio_postgres::NoTls;
use tokio_postgres::Client;
use tokio_postgres::Connection;
use tokio_postgres::Socket;

pub async fn init_db_client() -> anyhow::Result<(Client, Connection<Socket, NoTlsStream>)> {
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=postgres", NoTls).await?;
        Ok((client, connection))
}