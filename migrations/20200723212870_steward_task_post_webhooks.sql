CREATE TABLE IF NOT EXISTS steward_task_post_webhooks (
    id uuid NOT NULL,
    task_id uuid NOT NULL REFERENCES steward_tasks (id),
    created_at timestamp NOT NULL,
    updated_at timestamp NOT NULL,
    hook_url text NOT NULL,
    PRIMARY KEY (id)
);