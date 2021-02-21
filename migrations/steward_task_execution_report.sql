CREATE TABLE IF NOT EXISTS steward_task_execution_report (
    id uuid NOT NULL,
    task_id uuid NOT NULL REFERENCES steward_tasks (id),
    created_at timestamp NOT NULL,
    successful boolean NOT NULL,
    output text NOT NULL,
    PRIMARY KEY (id)
);