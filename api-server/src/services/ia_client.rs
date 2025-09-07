use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use thiserror::Error;

use crate::{
    config::IAConfig,
    handlers::microscope::{CaptureRequest, CaptureResponse, CommandResponse, MicroscopeStatus},
    models::MicroscopeCommand,
};

/// Client for communicating with IA system (OrangePi)
pub struct IAClient {
    client: Client,
    base_url: String,
    auth_token: Option<String>,
}

#[derive(Error, Debug)]
pub enum IAClientError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON serialization failed: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IA system returned error: {0}")]
    IAError(String),

    #[error("Invalid response from IA system")]
    InvalidResponse,
}

impl IAClient {
    /// Create new IA client
    pub fn new(config: &IAConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: config.base_url.clone(),
            auth_token: config.auth_token.clone(),
        }
    }

    /// Send command to microscope via IA system
    pub async fn send_command(
        &self,
        microscope_id: &str,
        command: &MicroscopeCommand,
    ) -> Result<CommandResponse, IAClientError> {
        let url = format!("{}/api/microscope/{}/command", self.base_url, microscope_id);

        let mut request = self.client.post(&url).json(command);

        if let Some(token) = &self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await?;

        if response.status().is_success() {
            let command_response: CommandResponse = response.json().await?;
            Ok(command_response)
        } else {
            let error_text = response.text().await?;
            Err(IAClientError::IAError(error_text))
        }
    }

    /// Get microscope status from IA system
    pub async fn get_status(&self, microscope_id: &str) -> Result<MicroscopeStatus, IAClientError> {
        let url = format!("{}/api/microscope/{}/status", self.base_url, microscope_id);

        let mut request = self.client.get(&url);

        if let Some(token) = &self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await?;

        if response.status().is_success() {
            let status: MicroscopeStatus = response.json().await?;
            Ok(status)
        } else {
            let error_text = response.text().await?;
            Err(IAClientError::IAError(error_text))
        }
    }

    /// Capture image from microscope
    pub async fn capture_image(
        &self,
        microscope_id: &str,
        request: &CaptureRequest,
    ) -> Result<CaptureResponse, IAClientError> {
        let url = format!("{}/api/microscope/{}/capture", self.base_url, microscope_id);

        let mut http_request = self.client.post(&url).json(request);

        if let Some(token) = &self.auth_token {
            http_request = http_request.header("Authorization", format!("Bearer {}", token));
        }

        let response = http_request.send().await?;

        if response.status().is_success() {
            let capture_response: CaptureResponse = response.json().await?;
            Ok(capture_response)
        } else {
            let error_text = response.text().await?;
            Err(IAClientError::IAError(error_text))
        }
    }

    /// Upload image metadata to IA system
    pub async fn upload_metadata(
        &self,
        microscope_id: &str,
        image_id: &str,
        metadata: &Value,
    ) -> Result<(), IAClientError> {
        let url = format!(
            "{}/api/microscope/{}/images/{}/metadata",
            self.base_url, microscope_id, image_id
        );

        let mut request = self.client.put(&url).json(metadata);

        if let Some(token) = &self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(IAClientError::IAError(error_text));
        }

        Ok(())
    }
}
