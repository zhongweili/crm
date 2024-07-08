use std::collections::HashMap;

use chrono::{DateTime, TimeZone, Utc};
use prost_types::Timestamp;
use sqlx::prelude::FromRow;
use tonic::{Response, Status};
use tracing::info;

use crate::{
    pb::{IdContent, QueryRequest, QueryRequestBuilder, RawQueryRequest, TimeQuery, User},
    ResponseStream, ServiceResult, UserStatsService,
};

#[derive(FromRow, Debug, Clone)]
struct UserModel {
    email: String,
    name: String,
    recent_watched: Vec<i32>,
    viewed_but_not_started: Vec<i32>,
    started_but_not_finished: Vec<i32>,
    finished: Vec<i32>,
}

impl UserStatsService {
    pub async fn query(&self, query: QueryRequest) -> ServiceResult<ResponseStream> {
        // generate sql based on query
        let mut sql = "SELECT * FROM user_stats WHERE ".to_string();

        let time_conditions = query
            .timestamps
            .into_iter()
            .map(|(k, v)| timestamp_query(&k, v.lower.as_ref(), v.upper.as_ref()))
            .collect::<Vec<String>>()
            .join(" AND ");

        sql.push_str(&time_conditions);

        let id_conditions = query
            .ids
            .into_iter()
            .map(|(k, v)| ids_query(&k, &v.ids))
            .collect::<Vec<String>>()
            .join(" AND ");

        if !id_conditions.is_empty() {
            sql.push_str(" AND ");
            sql.push_str(&id_conditions);
        }

        // the result list is too long, limit the result for convenience.
        sql.push_str(" LIMIT 100");

        info!("Generated SQL: {}", sql);

        self.raw_query(RawQueryRequest { query: sql }).await
    }

    pub async fn raw_query(&self, req: RawQueryRequest) -> ServiceResult<ResponseStream> {
        let Ok(ret) = sqlx::query_as::<_, UserModel>(&req.query)
            .fetch_all(&self.pool)
            .await
        else {
            return Err(Status::internal(format!(
                "Database error: Failed to execute query: {}",
                req.query
            )));
        };

        Ok(Response::new(Box::pin(futures::stream::iter(
            ret.into_iter()
                .map(|x| x.into_user())
                .collect::<Vec<User>>()
                .into_iter()
                .map(Ok),
        ))))
    }
}

impl UserModel {
    fn into_user(self: UserModel) -> User {
        let mut contents = HashMap::new();
        contents.insert(
            "recent_watched".to_string(),
            ids_to_content(self.recent_watched),
        );
        contents.insert(
            "viewed_but_not_started".to_string(),
            ids_to_content(self.viewed_but_not_started),
        );
        contents.insert(
            "started_but_not_finished".to_string(),
            ids_to_content(self.started_but_not_finished),
        );
        contents.insert("finished".to_string(), ids_to_content(self.finished));

        User {
            email: self.email,
            name: self.name,
            contents,
        }
    }
}

impl QueryRequest {
    pub fn new_with_dt(name: &str, lower: DateTime<Utc>, upper: DateTime<Utc>) -> Self {
        let ts = Timestamp {
            seconds: lower.timestamp(),
            nanos: 0,
        };
        let ts1 = Timestamp {
            seconds: upper.timestamp(),
            nanos: 0,
        };
        let tq = TimeQuery {
            lower: Some(ts),
            upper: Some(ts1),
        };

        QueryRequestBuilder::default()
            .timestamp((name.to_string(), tq))
            .build()
            .expect("Failed to build query request")
    }
}

fn ids_to_content(ids: Vec<i32>) -> IdContent {
    IdContent {
        ids: ids.iter().map(|&x| x as u32).collect::<Vec<u32>>(),
    }
}

fn ids_query(name: &str, ids: &[u32]) -> String {
    if ids.is_empty() {
        return "TRUE".to_string();
    }

    format!("array{:?} <@ {}", ids, name)
}

fn timestamp_query(name: &str, lower: Option<&Timestamp>, upper: Option<&Timestamp>) -> String {
    if lower.is_none() && upper.is_none() {
        return "TRUE".to_string();
    }

    if lower.is_none() {
        let upper = ts_to_utc(upper.unwrap());
        return format!("{} <= '{}'", name, upper.to_rfc3339());
    }

    if upper.is_none() {
        let lower = ts_to_utc(lower.unwrap());
        return format!("{} >= '{}'", name, lower.to_rfc3339());
    }

    format!(
        "{} BETWEEN '{}' AND '{}'",
        name,
        ts_to_utc(lower.unwrap()).to_rfc3339(),
        ts_to_utc(upper.unwrap()).to_rfc3339()
    )
}

fn ts_to_utc(ts: &Timestamp) -> DateTime<Utc> {
    Utc.timestamp_opt(ts.seconds, ts.nanos as _).unwrap()
}

#[cfg(test)]
mod test {

    use crate::{
        pb::QueryRequestBuilder,
        test_utils::{id, tq},
    };

    use super::*;
    use anyhow::Result;
    use futures::StreamExt;

    #[tokio::test]
    async fn raw_query_should_work() -> Result<()> {
        let (_tpg, service) = UserStatsService::new_for_test().await?;
        let req = RawQueryRequest {
            query: "SELECT * FROM user_stats WHERE created_at > '2024-01-01' limit 5".to_string(),
        };
        let mut stream = service.raw_query(req).await?.into_inner();

        while let Some(res) = stream.next().await {
            match res {
                Ok(user) => println!("{:?}", user),
                Err(e) => eprintln!("Error: {:?}", e),
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn query_should_work() -> Result<()> {
        let (_tpg, service) = UserStatsService::new_for_test().await?;
        let query = QueryRequestBuilder::default()
            .timestamp(("created_at".to_string(), tq(Some(120), None)))
            .timestamp(("last_visited_at".to_string(), tq(Some(30), None)))
            .id(("viewed_but_not_started".to_string(), id(&[252790])))
            .build()
            .unwrap();

        let mut stream = service.query(query).await?.into_inner();
        while let Some(res) = stream.next().await {
            match res {
                Ok(user) => println!("{:?}", user),
                Err(e) => eprintln!("Error: {:?}", e),
            }
        }

        Ok(())
    }
}
