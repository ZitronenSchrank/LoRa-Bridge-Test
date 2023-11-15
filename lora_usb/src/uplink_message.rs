use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RxInfoMetadata {
    pub region_common_name: String,
    pub region_config_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RxInfo {
    pub gateway_id: String,
    pub uplink_id: u64,
    pub time: String,
    pub rssi: i16,
    pub snr: f32,
    pub channel: u8,
    pub context: String,
    pub metadata: RxInfoMetadata,
    pub crc_status: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Lora {
    pub bandwidth: u64,
    pub spreading_factor: u8,
    pub code_rate: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Modulation {
    pub lora: Lora,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TxInfo {
    pub frequency: u64,
    pub modulation: Modulation,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub tenant_id: String,
    pub tenant_name: String,
    pub application_id: String,
    pub application_name: String,
    pub device_profile_id: String,
    pub device_profile_name: String,
    pub device_name: String,
    pub dev_eui: String,
    pub device_class_enabled: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UplinkMessage {
    pub deduplication_id: String,
    pub time: String,
    pub device_info: DeviceInfo,
    pub dev_addr: String,
    pub adr: bool,
    pub dr: u8,
    pub f_cnt: u64,
    pub f_port: u8,
    pub confirmed: bool,
    pub data: String,
    pub rx_info: Vec<RxInfo>,
    pub tx_info: TxInfo,
}
