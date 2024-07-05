-- Add migration script here
CREATE TYPE gender AS ENUM('male', 'female', 'unknown');
create table user_stats(
    email varchar(128) NOT NULL PRIMARY KEY,
    name varchar(64) NOT NULL,
    gender gender DEFAULT 'unknown',
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP,
    last_visited_at timestamptz,
    last_watched_at timestamptz,
    recent_watched int[],
    viewed_but_not_started int[],
    started_but_not_finished int[],
    finished int[],
    last_email_notification_at timestamptz,
    last_in_app_notification_at timestamptz,
    last_sms_notification_at timestamptz
);
