-- Add migration script here
create index user_stats_created_at_idx on user_stats(created_at);
create index user_stats_last_visited_at_idx on user_stats(last_visited_at);
create index user_stats_last_watched_at_idx on user_stats(last_watched_at);
create index user_stats_recent_watched_idx on user_stats using GIN(recent_watched);
create index user_stats_viewed_but_not_started_idx on user_stats using GIN(viewed_but_not_started);
create index user_stats_started_but_not_finished_idx on user_stats using GIN(started_but_not_finished);
create index user_stats_last_email_notification_idx on user_stats(last_email_notification_at);
create index user_stats_last_in_app_notification_at_idx on user_stats(last_in_app_notification_at);
create index user_stats_last_sms_notification_at_idx on user_stats(last_sms_notification_at);
