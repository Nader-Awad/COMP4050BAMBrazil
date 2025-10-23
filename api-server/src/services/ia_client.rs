use chrono::Utc;
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    config::IAConfig,
    handlers::microscope::{
        CaptureRequest, CaptureResponse, CommandResponse, FocusInfo, LightingInfo,
        MicroscopeStatus, Position,
    },
    models::{BoundingBox, DetectedObject, ImageMetadata, MicroscopeCommand},
};

/// Client for communicating with IA system (OrangePi)
pub struct IAClient {
    client: Client,
    base_url: String,
    auth_token: Option<String>,
    mock_mode: bool,
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
            mock_mode: config.mock_mode,
        }
    }

    /// Send command to microscope via IA system
    pub async fn send_command(
        &self,
        microscope_id: &str,
        command: &MicroscopeCommand,
    ) -> Result<CommandResponse, IAClientError> {
        // Return mock data if mock mode is enabled
        if self.mock_mode {
            return Ok(CommandResponse {
                success: true,
                message: Some(format!(
                    "Mock: Command {:?} executed on microscope {}",
                    command, microscope_id
                )),
                data: json!({
                    "command_id": format!("mock-cmd-{}", Uuid::new_v4()),
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }),
            });
        }

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
        // Return mock data if mock mode is enabled
        if self.mock_mode {
            return Ok(MicroscopeStatus {
                microscope_id: microscope_id.to_string(),
                is_connected: true,
                current_session: Some(Uuid::new_v4()),
                position: Position {
                    x: 150.5,
                    y: 220.3,
                    z: 45.8,
                },
                focus: FocusInfo {
                    is_focused: true,
                    focus_score: Some(0.89),
                    auto_focus_active: false,
                },
                magnification: "400x".to_string(),
                lighting: LightingInfo {
                    intensity: 75,
                    color_temperature: Some(5500),
                },
                tracking_active: false,
            });
        }

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
        // Return mock data if mock mode is enabled
        if self.mock_mode {
            let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
            let mock_objects = vec![
                DetectedObject {
                    class_name: "cell".to_string(),
                    confidence: 0.94,
                    bounding_box: BoundingBox {
                        x: 120.5,
                        y: 85.3,
                        width: 45.2,
                        height: 48.7,
                    },
                },
                DetectedObject {
                    class_name: "cell".to_string(),
                    confidence: 0.88,
                    bounding_box: BoundingBox {
                        x: 210.1,
                        y: 150.6,
                        width: 42.8,
                        height: 44.3,
                    },
                },
                DetectedObject {
                    class_name: "bacteria".to_string(),
                    confidence: 0.76,
                    bounding_box: BoundingBox {
                        x: 320.4,
                        y: 200.9,
                        width: 15.2,
                        height: 18.5,
                    },
                },
            ];

            return Ok(CaptureResponse {
                image_id: Uuid::new_v4(),
                filename: format!("microscope_{}_{}.jpg", microscope_id, timestamp),
                metadata: ImageMetadata {
                    objects_detected: mock_objects.clone(),
                    classification_tags: vec![
                        "biological_sample".to_string(),
                        "cells".to_string(),
                        "bacteria".to_string(),
                    ],
                    confidence_scores: mock_objects.iter().map(|obj| obj.confidence).collect(),
                    focus_quality: Some(0.91),
                    magnification: Some("400x".to_string()),
                    lighting_conditions: Some("optimal".to_string()),
                },
            });
        }

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

    /// Download image file from IA system
    pub async fn download_image(
        &self,
        microscope_id: &str,
        image_id: &Uuid,
    ) -> Result<Vec<u8>, IAClientError> {
        // Return mock image data if mock mode is enabled
        if self.mock_mode {
            tracing::info!(
                "Mock: Downloading image {} from microscope {}",
                image_id,
                microscope_id
            );
            // Return a minimal mock image (1x1 pixel JPEG)
            return Ok(vec![
                0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01,
                0x01, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xFF, 0xDB, 0x00, 0x43,
                0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01,
                0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0xFF, 0xC4, 0x00, 0x14, 0x00, 0x01,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x09, 0xFF, 0xC4, 0x00, 0x14, 0x10, 0x01, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0xFF, 0xDA, 0x00, 0x08, 0x01, 0x01, 0x00, 0x00, 0x3F, 0x00,
                0x7F, 0xC0, 0xFF, 0xD9,
            ]);
        }

        let url = format!(
            "{}/api/microscope/{}/images/{}",
            self.base_url, microscope_id, image_id
        );

        let mut request = self.client.get(&url);

        if let Some(token) = &self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await?;

        if response.status().is_success() {
            let bytes = response.bytes().await?;
            Ok(bytes.to_vec())
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
        // Return mock success if mock mode is enabled
        if self.mock_mode {
            // Simulate successful metadata upload
            tracing::info!(
                "Mock: Metadata uploaded for image {} on microscope {}",
                image_id,
                microscope_id
            );
            return Ok(());
        }

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

    /// Update microscope session status in IA system
    pub async fn update_session_status(
        &self,
        microscope_id: &str,
        session_id: Option<Uuid>,
        is_active: bool,
    ) -> Result<(), IAClientError> {
        // Return mock success if mock mode is enabled
        if self.mock_mode {
            tracing::info!(
                "Mock: Session status updated for microscope {} - session_id: {:?}, active: {}",
                microscope_id,
                session_id,
                is_active
            );
            return Ok(());
        }

        let url = format!("{}/api/microscope/{}/session", self.base_url, microscope_id);

        let payload = json!({
            "session_id": session_id,
            "is_active": is_active,
        });

        let mut request = self.client.put(&url).json(&payload);

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
