use bonsaidb::{
    core::{
        api::Infallible, async_trait::async_trait, connection::AsyncStorageConnection,
        keyvalue::AsyncKeyValue,
    },
    local::config::Builder,
    server::{
        api::{Handler, HandlerSession},
        Backend, CustomServer, DefaultPermissions, ServerConfiguration,
    },
};
use bonsaidb_counter_shared::{CounterValue, IncrementCounter, DATABASE_NAME};

/// The server's main entrypoint.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Open a `BonsaiDb` server at the given path, allowing all actions to be
    // done over the network connections.
    let server = CustomServer::<Example>::open(
        ServerConfiguration::new("counter-example.bonsaidb")
            .default_permissions(DefaultPermissions::AllowAll)
            .with_schema::<()>()?
            .with_api::<Example, CounterValue>()?
            .with_api::<Example, IncrementCounter>()?,
    )
    .await?;
    // Create the database if it doesn't exist.
    server.create_database::<()>(DATABASE_NAME, true).await?;
    // Start listening for websockets. This does not return until the server
    // shuts down. If you want to listen for multiple types of traffic, you will
    // need to spawn the tasks.
    server
        .listen_for_websockets_on("127.0.0.1:8081", false)
        .await?;

    Ok(())
}

/// The example database `Backend`.
#[derive(Debug)]
enum Example {}

impl Backend for Example {
    type ClientData = ();
    type Error = Infallible;
}

#[async_trait]
impl Handler<Example, CounterValue> for Example {
    async fn handle(
        session: HandlerSession<'_, Example>,
        _request: CounterValue,
    ) -> bonsaidb::server::api::HandlerResult<CounterValue> {
        println!("Returning current counter value.");
        let db = session.server.database::<()>(DATABASE_NAME).await.unwrap();

        let value = db
            .get_key("count")
            .into_u64()
            .await
            .unwrap()
            .unwrap_or_default();
        Ok(CounterValue(value))
    }
}

#[async_trait]
impl Handler<Example, IncrementCounter> for Example {
    /// Increments the counter, and publishes a message with the new value.
    async fn handle(
        session: HandlerSession<'_, Example>,
        _request: IncrementCounter,
    ) -> bonsaidb::server::api::HandlerResult<CounterValue> {
        let db = session.server.database::<()>(DATABASE_NAME).await?;

        let new_value = db.increment_key_by("count", 1_u64).await?;
        session
            .server
            .broadcast::<CounterValue>(&CounterValue(new_value))
            .await;

        Ok(CounterValue(new_value))
    }
}
