use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use lorewyld::{api::ApiServer, settings::Settings};
use std::path::PathBuf;
use tokio::signal;
use tracing_subscriber::{fmt::layer, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run().await
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg(short = 'c', long)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    cmd: Cmd,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let settings = Settings::new(self.config)?;
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new(&settings.log))
            .with(tracing_subscriber::fmt::layer())
            .init();

        self.cmd.run(settings).await
    }
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
    Server(Server),
}

impl Cmd {
    pub async fn run(&self, settings: Settings) -> Result<()> {
        match self {
            Self::Server(cmd) => cmd.run(&settings).await,
        }
    }
}

#[derive(Debug, Args)]
pub struct Server;

impl Server {
    pub async fn run(&self, settings: &Settings) -> Result<()> {
        let db_pool = settings.db_connect().await;
        sqlx::migrate!().run(&db_pool).await?;

        let shutdown_listener = shutdown_listener();

        let api_server = ApiServer::new(db_pool, settings.listen_addr()?);

        api_server.run(shutdown_listener).await
    }
}

fn shutdown_listener() -> triggered::Listener {
    let (trigger, listener) = triggered::trigger();
    let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
        .expect("Failed to initialize sigterm listener");
    tokio::spawn({
        async move {
            tokio::select! {
                _ = sigterm.recv() => trigger.trigger(),
                _ = signal::ctrl_c() => trigger.trigger(),
            }
        }
    });

    listener
}
