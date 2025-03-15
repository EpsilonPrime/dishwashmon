use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::auth::models::NestToken;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Device {
    pub name: String,          // Full path name
    pub device_id: String,     // Extracted ID from name
    pub type_name: String,     // Device type
    pub traits: Vec<String>,   // Device capabilities
    pub room_name: Option<String>, // Room location if available
    pub display_name: String,  // Human-friendly name
}

impl Device {
    // Extract the device ID from the full name path
    pub fn from_nest_device(device: NestDevice) -> Self {
        // Extract device ID from name (format: "enterprises/project-id/devices/device-id")
        let device_id = device.name
            .split('/')
            .last()
            .unwrap_or(&device.name)
            .to_string();
        
        let display_name = device
            .traits
            .get("info")
            .and_then(|info| info.get("customName"))
            .map(|name| name.as_str().unwrap_or("Unknown Camera"))
            .unwrap_or(&device_id)
            .to_string();

        let room_name = device
            .parent_relations
            .iter()
            .find(|rel| rel.relationship_type == "ROOM")
            .and_then(|rel| rel.display_name.clone());
            
        Self {
            name: device.name,
            device_id,
            type_name: device.type_name,
            traits: device.traits.keys().cloned().collect(),
            room_name,
            display_name,
        }
    }
}

#[derive(Debug, Deserialize)]
struct NestDevice {
    name: String,
    #[serde(rename = "type")]
    type_name: String,
    traits: serde_json::Map<String, serde_json::Value>,
    #[serde(default)]
    parent_relations: Vec<ParentRelation>,
}

#[derive(Debug, Deserialize)]
struct ParentRelation {
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    #[serde(rename = "relationshipType")]
    relationship_type: String,
}

#[derive(Debug, Deserialize)]
struct DevicesResponse {
    devices: Vec<NestDevice>,
}

/// Discover cameras and other devices for a user's project
pub async fn discover_devices(
    project_id: &str,
    token: &NestToken,
) -> Result<Vec<Device>, Box<dyn Error + Send + Sync>> {
    let client = Client::new();
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&format!("Bearer {}", token.access_token))?,
    );

    let url = format!(
        "https://smartdevicemanagement.googleapis.com/v1/enterprises/{}/devices",
        project_id
    );

    let response = client
        .get(&url)
        .headers(headers)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(format!("API error: {}", error_text).into());
    }

    let devices_response: DevicesResponse = response.json().await?;
    
    // Convert Nest devices to our Device struct
    let devices = devices_response.devices
        .into_iter()
        .map(Device::from_nest_device)
        .collect();

    Ok(devices)
}

/// Filter devices to only include cameras
pub fn filter_cameras(devices: &[Device]) -> Vec<Device> {
    devices
        .iter()
        .filter(|device| {
            device.type_name.contains("camera") || 
            device.traits.iter().any(|t| t.contains("camera"))
        })
        .cloned()
        .collect()
}