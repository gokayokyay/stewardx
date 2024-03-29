// use db::DBManager;
use executor::Executor;
use models::{
    OutputModel,
    // TaskModel
};
#[cfg(feature = "docker")]
use once_cell::sync::Lazy;

use reactor::Reactor;
#[cfg(feature = "server")]
use server::Server;
// use shiplift::Docker;
use std::{sync::Arc};
use tasks::{
    // CmdTask,
    // DockerImageType,
    // DockerTask,
    TaskWatcher
};
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{
    BunyanFormattingLayer,
    JsonStorageLayer
};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

mod db;
mod executor;
mod models;
mod reactor;
mod tasks;
mod traits;
mod types;
mod config;
#[cfg(target_os="linux")]
mod socket;
#[cfg(feature = "docker")]
static GLOBAL_DOCKER: Lazy<shiplift::Docker> = Lazy::new(|| shiplift::Docker::default());
mod server;

static CONFIG: Lazy<config::Config> = Lazy::new(|| config::Config::prepare_config());

#[tokio::main]
async fn main() {
    // Default filter
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::default());
    // Info filter - or RUST_LOG=stewardx cargo run
    // let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into()));
    let logs_dir = CONFIG.get_logs_folder_path();
    let file_appender = tracing_appender::rolling::daily(logs_dir, "stewardx.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let formatting_layer = BunyanFormattingLayer::new(
        "stewardx".into(),
        // Output the formatted spans to stdout.
        non_blocking,
    );
    let subscriber = Registry::default().with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    set_global_default(subscriber).expect("Failed to set subscriber");
    let pool = match db::connect(&std::env::var("STEWARDX_DATABASE_URL").unwrap()).await {
        Ok(p) => p,
        Err(_e) => {
            panic!("Database connection failed. Check if your connection URL is correct and your DB is reachable.")
        }
    };
    let (db_tx, db_rx) = tokio::sync::mpsc::channel(32);
    let (ex_tx, ex_rx) = tokio::sync::mpsc::channel(32);
    let (tw_tx, tw_rx) = tokio::sync::mpsc::channel(32);
    let (o_tx, mut o_rx) = tokio::sync::broadcast::channel::<OutputModel>(128);
    #[cfg(feature = "server")]
    let (sv_tx, sv_rx) = tokio::sync::mpsc::channel(32);
    #[cfg(not(feature = "server"))]
    let (_sv_tx, sv_rx) = tokio::sync::mpsc::channel(32);
    
    tokio::spawn(async {
        // let task = CmdTask::new(uuid::Uuid::new_v4(), Box::new("/bin/bash temp.sh".to_string()));
        // DBManager::create_task(&mut pool.acquire().await.unwrap(), TaskModel::from_boxed_task(Box::new(task), "testing".into(), "Every(30 * * * * * *)".to_string())).await;
        // let docker_task = DockerTask::new(uuid::Uuid::new_v4(), DockerImageType::File(format!("from alpine:latest\ncmd sleep 15 && echo 'hello'")), Vec::default());
        // DBManager::create_task(&mut pool.acquire().await.unwrap(), TaskModel::from_boxed_task(Box::new(docker_task), "gokay-testx-file".into(), "Every(45 * * * * * *)".to_string())).await;
        let mut db_manager = db::DBManager::new(pool, db_rx);
        db_manager.listen().await;
    });
    let inner_ex_tx = ex_tx.clone();
    tokio::spawn(async {
        let mut executor = Executor {
            task_handles: Vec::default(),
        };
        executor.listen(ex_rx, inner_ex_tx).await;
    });

    tokio::spawn(async move {
        while let Ok(output) = o_rx.recv().await {
            println!("{:?}", output);
        }
    });

    tokio::spawn(async move {
        let task_watcher = TaskWatcher {};
        task_watcher.listen(tw_rx).await;
    });

    #[cfg(feature = "server")]
    tokio::spawn(async move {
        let server = Server::new(sv_tx);
        let port = (std::env::var("STEWARDX_SERVER_PORT").unwrap_or("3000".to_string())).parse::<i64>().expect("STEWARDX_SERVER_PORT is not a number?");
        let host = std::env::var("STEWARDX_SERVER_HOST").unwrap_or("127.0.0.1".to_string());
        server.listen(host, port).await;
    });

    tokio::spawn(async move {
        if cfg!(unix) {
            use tokio::signal::unix::*;
            let mut hup = signal(SignalKind::hangup()).unwrap();
            let mut int = signal(SignalKind::interrupt()).unwrap();
            let mut quit = signal(SignalKind::quit()).unwrap();
            let mut term = signal(SignalKind::terminate()).unwrap();

            tokio::select! {
                v = hup.recv() => v.unwrap(),
                v = int.recv() => v.unwrap(),
                v = quit.recv() => v.unwrap(),
                v = term.recv() => v.unwrap(),
            }

            println!("Goodbye!");
            socket::unix_utils::exit();
        }
    });
    tokio::spawn(async move {
        socket::SocketManager::listen().await;
    });
    let _ = tokio::spawn(async {
        let server_receiver = Arc::new(tokio::sync::Mutex::new(sv_rx));
        let (tx, rx) = tokio::sync::mpsc::channel(128);
        let mut reactor = Reactor {
            db_sender: db_tx,
            executor_sender: ex_tx,
            task_watcher_sender: tw_tx,
            output_emitter: o_tx,
            server_receiver,
            inner_sender: tx,
        };
        reactor.listen(rx).await;
    })
    .await;
}
