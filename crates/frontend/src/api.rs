// This starter keeps browser gRPC-Web plumbing deliberately local to this module.
// Yew callers get small operations (`health_check`, `create_event`) while this file
// owns the HTTP request details, protobuf bytes, gRPC-Web frames, and trailers.
use gloo_net::http::Request;
use prost::Message;
use shared::app::{
    CreateEventRequest, CreateEventResponse, HealthCheckRequest, HealthCheckResponse,
};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ApiError {
    #[error("request failed: {0}")]
    Request(String),
    #[error("gRPC-Web response was invalid")]
    InvalidResponse,
    #[error("gRPC status {code}: {message}")]
    GrpcStatus { code: u32, message: String },
    #[error("decode failed: {0}")]
    Decode(#[from] prost::DecodeError),
}

pub async fn health_check(base_url: &str) -> Result<String, ApiError> {
    let response: HealthCheckResponse = grpc_web_unary(
        base_url,
        "/app.v1.AppService/HealthCheck",
        HealthCheckRequest {},
    )
    .await?;
    Ok(response.status)
}

pub async fn create_event(
    base_url: &str,
    name: String,
    payload: String,
) -> Result<String, ApiError> {
    let response: CreateEventResponse = grpc_web_unary(
        base_url,
        "/app.v1.AppService/CreateEvent",
        CreateEventRequest { name, payload },
    )
    .await?;
    Ok(response.id)
}

async fn grpc_web_unary<Req, Resp>(
    base_url: &str,
    path: &str,
    message: Req,
) -> Result<Resp, ApiError>
where
    Req: Message,
    Resp: Message + Default,
{
    let mut body = Vec::new();
    let mut payload = Vec::new();
    message
        .encode(&mut payload)
        .expect("encoding into Vec cannot fail");
    body.push(0);
    body.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    body.extend_from_slice(&payload);

    let bytes = Request::post(&format!("{}{}", base_url.trim_end_matches('/'), path))
        .header("content-type", "application/grpc-web+proto")
        .header("x-grpc-web", "1")
        .body(body)
        .map_err(|error| ApiError::Request(error.to_string()))?
        .send()
        .await
        .map_err(|error| ApiError::Request(error.to_string()))?
        .binary()
        .await
        .map_err(|error| ApiError::Request(error.to_string()))?;

    decode_grpc_web_response::<Resp>(&bytes)
}

const GRPC_WEB_TRAILER_FLAG: u8 = 0x80;

#[derive(Default)]
struct GrpcWebResult<Resp> {
    message: Option<Resp>,
    status: Option<u32>,
    status_message: String,
}

fn decode_grpc_web_response<Resp>(bytes: &[u8]) -> Result<Resp, ApiError>
where
    Resp: Message + Default,
{
    let mut result = GrpcWebResult::default();

    for frame in GrpcWebFrames::new(bytes) {
        let (flags, frame) = frame?;
        if flags & GRPC_WEB_TRAILER_FLAG == 0 {
            result.message = Some(Resp::decode(frame)?);
        } else {
            parse_grpc_web_trailers(frame, &mut result)?;
        }
    }

    match result.status {
        Some(0) => result.message.ok_or(ApiError::InvalidResponse),
        Some(code) => Err(ApiError::GrpcStatus {
            code,
            message: result.status_message,
        }),
        None => Err(ApiError::InvalidResponse),
    }
}

struct GrpcWebFrames<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> GrpcWebFrames<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }
}

impl<'a> Iterator for GrpcWebFrames<'a> {
    type Item = Result<(u8, &'a [u8]), ApiError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset == self.bytes.len() {
            return None;
        }

        let header = match self.bytes.get(self.offset..self.offset + 5) {
            Some(header) => header,
            None => {
                self.offset = self.bytes.len();
                return Some(Err(ApiError::InvalidResponse));
            }
        };

        let flags = header[0];
        let len = u32::from_be_bytes([header[1], header[2], header[3], header[4]]) as usize;
        let payload_start = self.offset + 5;
        let payload_end = match payload_start.checked_add(len) {
            Some(end) => end,
            None => {
                self.offset = self.bytes.len();
                return Some(Err(ApiError::InvalidResponse));
            }
        };
        let payload = match self.bytes.get(payload_start..payload_end) {
            Some(payload) => payload,
            None => {
                self.offset = self.bytes.len();
                return Some(Err(ApiError::InvalidResponse));
            }
        };

        self.offset = payload_end;
        Some(Ok((flags, payload)))
    }
}

fn parse_grpc_web_trailers<Resp>(
    frame: &[u8],
    result: &mut GrpcWebResult<Resp>,
) -> Result<(), ApiError> {
    let trailers = std::str::from_utf8(frame).map_err(|_| ApiError::InvalidResponse)?;
    for line in trailers.lines() {
        if let Some(value) = line.strip_prefix("grpc-status:") {
            result.status = Some(
                value
                    .trim()
                    .parse::<u32>()
                    .map_err(|_| ApiError::InvalidResponse)?,
            );
        } else if let Some(value) = line.strip_prefix("grpc-message:") {
            result.status_message = percent_decode(value.trim())?;
        }
    }
    Ok(())
}

fn percent_decode(value: &str) -> Result<String, ApiError> {
    if !value.as_bytes().contains(&b'%') {
        return Ok(value.to_string());
    }

    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return Err(ApiError::InvalidResponse);
            }
            let high = hex_value(bytes[index + 1]).ok_or(ApiError::InvalidResponse)?;
            let low = hex_value(bytes[index + 2]).ok_or(ApiError::InvalidResponse)?;
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }

    String::from_utf8(decoded).map_err(|_| ApiError::InvalidResponse)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::app::HealthCheckResponse;

    fn frame(flags: u8, payload: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(flags);
        bytes.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        bytes.extend_from_slice(payload);
        bytes
    }

    fn encoded_health_response() -> Vec<u8> {
        let mut payload = Vec::new();
        HealthCheckResponse {
            status: "ok".to_string(),
        }
        .encode(&mut payload)
        .unwrap();
        payload
    }

    #[test]
    fn decodes_successful_grpc_web_response() {
        let mut bytes = frame(0, &encoded_health_response());
        bytes.extend(frame(0x80, b"grpc-status: 0\r\n"));

        let response = decode_grpc_web_response::<HealthCheckResponse>(&bytes).unwrap();

        assert_eq!(response.status, "ok");
    }

    #[test]
    fn returns_grpc_status_errors_from_trailers() {
        let bytes = frame(
            0x80,
            b"grpc-status: 3\r\ngrpc-message: invalid%20name%3A%20%C3%BC\r\n",
        );

        let error = decode_grpc_web_response::<HealthCheckResponse>(&bytes).unwrap_err();

        assert_eq!(
            error,
            ApiError::GrpcStatus {
                code: 3,
                message: "invalid name: ü".to_string(),
            }
        );
    }

    #[test]
    fn rejects_malformed_grpc_web_responses() {
        let bytes = [0, 0, 0, 0, 5, 1, 2];

        let error = decode_grpc_web_response::<HealthCheckResponse>(&bytes).unwrap_err();

        assert_eq!(error, ApiError::InvalidResponse);
    }
}
