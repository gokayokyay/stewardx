CREATE TABLE IF NOT EXISTS steward_task_errors (
    id uuid NOT NULL,
    task_id uuid NOT NULL REFERENCES steward_tasks (id),
    created_at timestamp NOT NULL,
    error_type text NOT NULL,
    error_message text NOT NULL,
    PRIMARY KEY (id)
)