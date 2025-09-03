use bzd_lib::error::Error;

mod app;

#[tokio::main]
async fn main() -> Result<(), Error> {
    bzd_lib::tracing::init()?;

    app::run().await?;

    Ok(())
}
