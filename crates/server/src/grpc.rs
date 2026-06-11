use std::sync::Arc;

use shared::app::{
    app_service_server::AppService, CreateEventRequest, CreateEventResponse, HealthCheckRequest,
    HealthCheckResponse,
};
use tonic::{Request, Response, Status};

use crate::events::{EventName, EventRepository};

#[derive(Clone)]
pub struct AppGrpcService {
    events: Arc<EventRepository>,
}

impl AppGrpcService {
    pub fn new(events: Arc<EventRepository>) -> Self {
        Self { events }
    }
}

#[tonic::async_trait]
impl AppService for AppGrpcService {
    async fn health_check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        Ok(Response::new(HealthCheckResponse {
            status: "ok".to_string(),
        }))
    }

    async fn create_event(
        &self,
        request: Request<CreateEventRequest>,
    ) -> Result<Response<CreateEventResponse>, Status> {
        let request = request.into_inner();
        let name = EventName::parse(&request.name)
            .map_err(|_| Status::invalid_argument("event name must not be empty"))?;

        let id = self
            .events
            .create_event(name, &request.payload)
            .await
            .map_err(|error| Status::internal(format!("failed to create event: {error}")))?;

        Ok(Response::new(CreateEventResponse { id: id.to_string() }))
    }
}
