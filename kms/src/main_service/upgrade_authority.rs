use crate::config::AuthApi;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_human_bytes as hex_bytes;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BootInfo {
    #[serde(with = "hex_bytes")]
    pub mrtd: Vec<u8>,
    #[serde(with = "hex_bytes")]
    pub rtmr0: Vec<u8>,
    #[serde(with = "hex_bytes")]
    pub rtmr1: Vec<u8>,
    #[serde(with = "hex_bytes")]
    pub rtmr2: Vec<u8>,
    #[serde(with = "hex_bytes")]
    pub rtmr3: Vec<u8>,
    #[serde(with = "hex_bytes")]
    pub mr_aggregated: Vec<u8>,
    #[serde(with = "hex_bytes")]
    pub os_image_hash: Vec<u8>,
    #[serde(with = "hex_bytes")]
    pub mr_system: Vec<u8>,
    #[serde(with = "hex_bytes")]
    pub app_id: Vec<u8>,
    #[serde(with = "hex_bytes")]
    pub compose_hash: Vec<u8>,
    #[serde(with = "hex_bytes")]
    pub instance_id: Vec<u8>,
    #[serde(with = "hex_bytes")]
    pub device_id: Vec<u8>,
    #[serde(with = "hex_bytes")]
    pub key_provider_info: Vec<u8>,
    pub event_log: String,
    pub tcb_status: String,
    pub advisory_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BootResponse {
    pub is_allowed: bool,
    pub gateway_app_id: String,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuthApiInfoResponse {
    pub status: String,
    pub kms_contract_addr: String,
    pub gateway_app_id: String,
    pub chain_id: u64,
    pub app_implementation: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetInfoResponse {
    pub is_dev: bool,
    pub gateway_app_id: Option<String>,
    pub kms_contract_address: Option<String>,
    pub chain_id: Option<u64>,
    pub app_implementation: Option<String>,
}

impl AuthApi {
    pub async fn is_app_allowed(&self, boot_info: &BootInfo, is_kms: bool) -> Result<BootResponse> {
        match self {
            AuthApi::Dev { dev } => Ok(BootResponse {
                is_allowed: true,
                reason: "".to_string(),
                gateway_app_id: dev.gateway_app_id.clone(),
            }),
            AuthApi::Webhook { webhook } => {
                let client = reqwest::Client::new();
                let path = if is_kms {
                    "bootAuth/kms"
                } else {
                    "bootAuth/app"
                };
                let url = url_join(&webhook.url, path);
                let response = client.post(&url).json(&boot_info).send().await?;
                if !response.status().is_success() {
                    bail!("Failed to check boot auth: {}", response.text().await?);
                }
                Ok(response.json().await?)
            }
        }
    }

    pub async fn get_info(&self) -> Result<GetInfoResponse> {
        match self {
            AuthApi::Dev { dev } => Ok(GetInfoResponse {
                is_dev: true,
                kms_contract_address: None,
                gateway_app_id: Some(dev.gateway_app_id.clone()),
                chain_id: None,
                app_implementation: None,
            }),
            AuthApi::Webhook { webhook } => {
                let client = reqwest::Client::new();
                let response = client.get(&webhook.url).send().await?;
                println!("url: {}", webhook.url);
                let info: AuthApiInfoResponse = response.json().await?;
                Ok(GetInfoResponse {
                    is_dev: false,
                    kms_contract_address: Some(info.kms_contract_addr.clone()),
                    chain_id: Some(info.chain_id),
                    gateway_app_id: Some(info.gateway_app_id.clone()),
                    app_implementation: Some(info.app_implementation.clone()),
                })
            }
        }
    }
}

fn url_join(url: &str, path: &str) -> String {
    let mut url = url.to_string();
    if !url.ends_with('/') {
        url.push('/');
    }
    url.push_str(path);
    url
}
