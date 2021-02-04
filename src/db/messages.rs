use uuid::Uuid;

#[derive(Debug)]
pub enum DBMessage {
    GET {
        id: Uuid,
        // resp: DBMessageResponse<BoxedTask>
    },
    CREATE {
        // task: BoxedTask,
        frequency: String,
        resp: DBMessageResponse<Uuid>
    }
}

pub type DBMessageResponse<T> =
    tokio::sync::oneshot::Sender<T>;
