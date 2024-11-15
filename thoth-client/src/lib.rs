mod parameters;
// GraphQLQuery derive macro breaks this linting rule - ignore while awaiting fix
#[allow(clippy::derive_partial_eq_without_eq)]
mod queries;

pub use crate::parameters::QueryParameters;
use crate::parameters::{WorkQueryVariables, WorksQueryVariables};
pub use crate::queries::work_query::*;
use crate::queries::{
    work_count_query, work_last_updated_query, work_query, works_last_updated_query, works_query,
    WorkCountQuery, WorkLastUpdatedQuery, WorkQuery, WorksLastUpdatedQuery, WorksQuery,
};
pub use chrono::NaiveDate;
use graphql_client::GraphQLQuery;
use graphql_client::Response;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::Serialize;
use std::future::Future;
use thoth_api::model::Timestamp;
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

/// Maximum number of allowed request retries attempts.
const MAX_REQUEST_RETRIES: u32 = 5;

type HttpFuture = Result<reqwest::Response, reqwest_middleware::Error>;

/// A GraphQL `ThothClient` to query metadata
pub struct ThothClient {
    graphql_endpoint: String,
    http_client: ClientWithMiddleware,
}

impl ThothClient {
    /// Constructs a new `ThothClient`
    pub fn new(graphql_endpoint: String) -> Self {
        let retry_policy =
            ExponentialBackoff::builder().build_with_max_retries(MAX_REQUEST_RETRIES);
        let http_client = ClientBuilder::new(reqwest::Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();
        ThothClient {
            graphql_endpoint,
            http_client,
        }
    }

    async fn post_request<T: Serialize + ?Sized>(
        &self,
        request_body: &T,
    ) -> impl Future<Output = HttpFuture> {
        self.http_client
            .post(&self.graphql_endpoint)
            .json(&request_body)
            .send()
    }

    /// Get a `Work` from Thoth given its `work_id`
    ///
    /// # Errors
    ///
    /// This method fails if the `work_id` was not found
    /// or if there was an error while sending the request
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use thoth_errors::ThothResult;
    /// # use thoth_client::{QueryParameters, ThothClient, Work};
    /// # use uuid::Uuid;
    ///
    /// # async fn run() -> ThothResult<Work> {
    /// let thoth_client = ThothClient::new("https://api.thoth.pub/graphql".to_string());
    /// let work_id = Uuid::parse_str("00000000-0000-0000-AAAA-000000000001")?;
    /// let work = thoth_client.get_work(work_id, QueryParameters::new()).await?;
    /// # Ok(work)
    /// # }
    /// ```
    pub async fn get_work(&self, work_id: Uuid, parameters: QueryParameters) -> ThothResult<Work> {
        let variables: work_query::Variables = WorkQueryVariables::new(work_id, parameters).into();
        let request_body = WorkQuery::build_query(variables);
        let res = self.post_request(&request_body).await.await?;
        let response_body: Response<work_query::ResponseData> = res.json().await?;
        match response_body.data {
            Some(data) => Ok(data.work),
            None => Err(ThothError::EntityNotFound),
        }
    }

    /// Get a list of `Work`s from Thoth
    ///
    /// # Errors
    ///
    /// This method fails if there was an error while sending the request
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use thoth_errors::ThothResult;
    /// # use thoth_client::{QueryParameters, ThothClient, Work};
    /// # use uuid::Uuid;
    ///
    /// # async fn run() -> ThothResult<Vec<Work>> {
    /// let thoth_client = ThothClient::new("https://api.thoth.pub/graphql".to_string());
    /// let publisher_id = Uuid::parse_str("00000000-0000-0000-AAAA-000000000001")?;
    /// let works = thoth_client.get_works(Some(vec![publisher_id]), 100, 0, QueryParameters::new()).await?;
    /// # Ok(works)
    /// # }
    /// ```
    pub async fn get_works(
        &self,
        publishers: Option<Vec<Uuid>>,
        limit: i64,
        offset: i64,
        parameters: QueryParameters,
    ) -> ThothResult<Vec<Work>> {
        let variables: works_query::Variables =
            WorksQueryVariables::new(publishers, limit, offset, parameters).into();
        let request_body = WorksQuery::build_query(variables);
        let res = self.post_request(&request_body).await.await?;
        let response_body: Response<works_query::ResponseData> = res.json().await?;
        match response_body.data {
            Some(data) => Ok(data.works.iter().map(|w| w.clone().into()).collect()), // convert works_query::Work into work_query::Work
            None => Err(ThothError::EntityNotFound),
        }
    }

    /// Get the number of `Work`s in Thoth
    ///
    /// # Errors
    ///
    /// This method fails if there was an error while sending the request
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use thoth_errors::ThothResult;
    /// # use thoth_client::ThothClient;
    /// # use uuid::Uuid;
    ///
    /// # async fn run() -> ThothResult<i64> {
    /// let thoth_client = ThothClient::new("https://api.thoth.pub/graphql".to_string());
    /// let publisher_id = Uuid::parse_str("00000000-0000-0000-AAAA-000000000001")?;
    /// let work_count = thoth_client.get_work_count(Some(vec![publisher_id])).await?;
    /// # Ok(work_count)
    /// # }
    /// ```
    pub async fn get_work_count(&self, publishers: Option<Vec<Uuid>>) -> ThothResult<i64> {
        let variables = work_count_query::Variables { publishers };
        let request_body = WorkCountQuery::build_query(variables);
        let res = self.post_request(&request_body).await.await?;
        let response_body: Response<work_count_query::ResponseData> = res.json().await?;
        match response_body.data {
            Some(data) => Ok(data.work_count),
            None => Err(ThothError::EntityNotFound),
        }
    }

    /// Get the `updated_at_with_relations` date of a `Work` in Thoth
    ///
    /// # Errors
    ///
    /// This method fails if there was an error while sending the request
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use thoth_api::model::Timestamp;
    /// # use thoth_errors::ThothResult;
    /// # use thoth_client::{ThothClient, NaiveDate};
    /// # use uuid::Uuid;
    ///
    /// # async fn run() -> ThothResult<Timestamp> {
    /// let thoth_client = ThothClient::new("https://api.thoth.pub/graphql".to_string());
    /// let publisher_id = Uuid::parse_str("00000000-0000-0000-AAAA-000000000001")?;
    /// let work_last_updated = thoth_client.get_work_last_updated(publisher_id).await?;
    /// # Ok(work_last_updated)
    /// # }
    /// ```
    pub async fn get_work_last_updated(&self, work_id: Uuid) -> ThothResult<Timestamp> {
        let variables = work_last_updated_query::Variables { work_id };
        let request_body = WorkLastUpdatedQuery::build_query(variables);
        let res = self.post_request(&request_body).await.await?;
        let response_body: Response<work_last_updated_query::ResponseData> = res.json().await?;
        match response_body.data {
            Some(data) => Ok(data.work.updated_at_with_relations),
            None => Err(ThothError::EntityNotFound),
        }
    }

    /// Get the last `updated_at_with_relations` date of published `Work`s in Thoth
    ///
    /// # Errors
    ///
    /// This method fails if there was an error while sending the request
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use thoth_api::model::Timestamp;
    /// # use thoth_errors::ThothResult;
    /// # use thoth_client::{ThothClient, NaiveDate};
    /// # use uuid::Uuid;
    ///
    /// # async fn run() -> ThothResult<Timestamp> {
    /// let thoth_client = ThothClient::new("https://api.thoth.pub/graphql".to_string());
    /// let publisher_id = Uuid::parse_str("00000000-0000-0000-AAAA-000000000001")?;
    /// let work_last_updated = thoth_client.get_works_last_updated(Some(vec![publisher_id])).await?;
    /// # Ok(work_last_updated)
    /// # }
    /// ```
    pub async fn get_works_last_updated(
        &self,
        publishers: Option<Vec<Uuid>>,
    ) -> ThothResult<Timestamp> {
        let variables = works_last_updated_query::Variables { publishers };
        let request_body = WorksLastUpdatedQuery::build_query(variables);
        let res = self.post_request(&request_body).await.await?;
        let response_body: Response<works_last_updated_query::ResponseData> = res.json().await?;
        match response_body.data {
            Some(data) => {
                if let Some(work) = data.works.first() {
                    Ok(work.updated_at_with_relations)
                } else {
                    Err(ThothError::EntityNotFound)
                }
            }
            None => Err(ThothError::EntityNotFound),
        }
    }
}
