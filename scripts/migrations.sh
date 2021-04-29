psql $STEWARDX_DATABASE_URL -f ../migrations/20200723212810_steward_tasks.sql
psql $STEWARDX_DATABASE_URL -f ../migrations/20200723212830_steward_task_errors.sql
psql $STEWARDX_DATABASE_URL -f ../migrations/20200723212850_steward_task_execution_report.sql