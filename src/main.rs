use db::DBManager;
use executor::Executor;
use models::{OutputModel, TaskModel};
use reactor::Reactor;
use server::Server;
use tokio::task::spawn_blocking;
use std::{str::FromStr, sync::Arc};
use tasks::{CmdTask, TaskWatcher};
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

mod db;
mod executor;
mod models;
mod reactor;
mod server;
mod tasks;
mod traits;
mod types;

#[tokio::main]
async fn main() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new(
        "stewardx".into(),
        // Output the formatted spans to stdout.
        std::io::stdout,
    );
    let subscriber = Registry::default().with(env_filter);
    // .with(JsonStorageLayer)
    // .with(formatting_layer);
    set_global_default(subscriber).expect("Failed to set subscriber");
    let pool = db::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    let (db_tx, db_rx) = tokio::sync::mpsc::channel(32);
    let (ex_tx, ex_rx) = tokio::sync::mpsc::channel(32);
    let (tw_tx, tw_rx) = tokio::sync::mpsc::channel(32);
    let (o_tx, mut o_rx) = tokio::sync::broadcast::channel::<OutputModel>(128);
    let (sv_tx, mut sv_rx) = tokio::sync::mpsc::channel(32);

    tokio::spawn(async {
        // let task = CmdTask::new(uuid::Uuid::new_v4(), Box::new("/bin/bash temp.sh".to_string()));
        // DBManager::create_task(&mut pool.acquire().await.unwrap(), TaskModel::from_boxed_task(Box::new(task), "Every(30 * * * * * *)".to_string())).await;
        let mut db_manager = db::DBManager::new(pool, db_rx);
        db_manager.listen().await;
    });
    tokio::spawn(async {
        let executor = Executor {};
        executor.listen(ex_rx).await;
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

    tokio::spawn(async move {
        let server = Server::new(sv_tx);
        server.listen(String::from("0.0.0.0"), 3000).await;
    });

    tokio::spawn(async {
        let server_receiver = Arc::new(tokio::sync::Mutex::new(sv_rx));
        let mut reactor = Reactor {
            db_sender: db_tx,
            executor_sender: ex_tx,
            task_watcher_sender: tw_tx,
            output_emitter: o_tx,
            server_receiver,
        };
        reactor.listen().await;
    })
    .await;
    // let dbman = db::DBManager::new(pool, db_rx);
    // dbman.delete_task(uuid::Uuid::from_str("6c76b64f-6497-43dd-bd2b-05a1931164bb").unwrap()).await;
    // let task = CmdTask::new(uuid::Uuid::new_v4(), Box::new("cat Cargo.toml".to_string()));
    // DBManager::create_task(&mut pool.acquire().await.unwrap(), TaskModel::from_boxed_task(Box::new(task), "Every(30 * * * * * *)".to_string())).await;
    // // // let t = dbman.get_task(uuid::Uuid::from_str("9de07abf-7832-4ae2-be61-793912bb5805").unwrap()).await;
    // // // println!("{:?}", t);
    // dbman.create_task(TaskModel::from_boxed_task(Box::new(task), "Every(30 * * * * * *)".to_string())).await;
}
