use std::str::FromStr;

use executor::Executor;
use models::TaskModel;
use reactor::Reactor;
use tasks::CmdTask;

mod traits;
mod types;
mod db;
mod tasks;
mod models;
mod reactor;
mod executor;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let pool = db::connect(&std::env::var("DATABASE_URL").unwrap()).await.unwrap();
    let (db_tx, db_rx) = tokio::sync::mpsc::channel(32);
    let (ex_tx, ex_rx) = tokio::sync::mpsc::channel(32);
    // tokio::spawn(async {
    //     let mut db_manager = db::DBManager::new(pool, db_rx);
    //     db_manager.listen().await;
    // });
    // tokio::spawn(async {
    //     let executor = Executor {};
    //     executor.listen(ex_rx).await;
    // });
    // tokio::spawn(async {
    //     let mut reactor = Reactor { db_sender: db_tx, executor_sender: ex_tx};
    //     reactor.listen().await;
    // }).await;
    let dbman = db::DBManager::new(pool, db_rx);
    let task = CmdTask::new(uuid::Uuid::new_v4(), Box::new("cat Cargo.lock".to_string()));
    // dbman.create_task(Box::new(task), "Every(30 * * * * * *)".to_string()).await;
    // // let t = dbman.get_task(uuid::Uuid::from_str("9de07abf-7832-4ae2-be61-793912bb5805").unwrap()).await;
    // // println!("{:?}", t);
    dbman.create_task(TaskModel::from_boxed_task(Box::new(task), "Every(30 * * * * * *)".to_string())).await;
}
