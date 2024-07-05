use chrono::{DateTime, TimeZone, Utc};
use prost_types::Timestamp;
use tonic::{Response, Status};

use crate::{
    pb::{QueryRequest, RawQueryRequest, User},
    ResponseStream, ServiceResult, UserStatsService,
};

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
        self.raw_query(RawQueryRequest { query: sql }).await
    }

    pub async fn raw_query(&self, req: RawQueryRequest) -> ServiceResult<ResponseStream> {
        let Ok(ret) = sqlx::query_as::<_, User>(&req.query)
            .fetch_all(&self.pool)
            .await
        else {
            return Err(Status::internal(format!(
                "Database error: Failed to execute query: {}",
                req.query
            )));
        };

        Ok(Response::new(Box::pin(futures::stream::iter(
            ret.into_iter().map(Ok),
        ))))
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

    use crate::pb::{IdQuery, QueryRequestBuilder, TimeQuery};

    use super::*;
    use anyhow::Result;
    use futures::StreamExt;

    #[tokio::test]
    async fn raw_query_should_work() -> Result<()> {
        let service = UserStatsService::new().await;
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
        let service = UserStatsService::new().await;
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

    pub fn tq(lower: Option<i64>, upper: Option<i64>) -> TimeQuery {
        TimeQuery {
            lower: lower.map(to_ts),
            upper: upper.map(to_ts),
        }
    }

    pub fn to_ts(days: i64) -> Timestamp {
        let dt = Utc
            .with_ymd_and_hms(2024, 5, 7, 0, 0, 0)
            .unwrap()
            .checked_sub_signed(chrono::Duration::days(days))
            .unwrap();
        Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        }
    }
    pub fn id(id: &[u32]) -> IdQuery {
        IdQuery { ids: id.to_vec() }
    }
}
