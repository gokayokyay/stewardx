CREATE TABLE IF NOT EXISTS steward_tasks (
    id uuid NOT NULL,
    created_at timestamp NOT NULL,
    updated_at timestamp NOT NULL,
    task_type varchar(30) NOT NULL,
    serde_string varchar NOT NULL,
    frequency text NOT NULL,
    interval bigint,
    last_execution timestamp,
    next_execution timestamp,
    last_exec_succeeded boolean NOT NULL,
    exec_count bigint DEFAULT '0' NOT NULL,
    PRIMARY KEY (id)
);