use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct GPUDeviceOverrides {
    #[serde(default)]
    pub device_overrides: DeviceOverrideConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct DeviceOverrideConfig {
    #[serde(default)]
    pub integrated_gpus: Vec<String>,
}

static GPU_OVERRIDES: OnceLock<GPUDeviceOverrides> = OnceLock::new();
static INTEGRATED_GPU_SET: OnceLock<HashSet<String>> = OnceLock::new();

impl GPUDeviceOverrides {
    const PLATFORM_DIR: &str = "/usr/share/powerstation/platform";
    const GPU_OVERRIDES_FILE: &str = "gpu_device_overrides.toml";

    fn load() -> Self {
        let config_path = Path::new(Self::PLATFORM_DIR).join(Self::GPU_OVERRIDES_FILE);

        match fs::read_to_string(&config_path) {
            Ok(config_str) => toml::from_str(&config_str).unwrap_or_else(|e| {
                log::warn!("Failed to parse GPU overrides config: {}", e);
                Self::default()
            }),
            Err(_) => {
                log::debug!("No GPU device overrides found, using defaults");
                Self::default()
            }
        }
    }

    /// Check if device should be treated as integrated GPU
    fn is_integrated_device(&self, vendor_id: &str, device_id: &str) -> bool {
        let device_key = format!("{}:{}", vendor_id, device_id);
        let integrated_set = INTEGRATED_GPU_SET.get_or_init(|| {
            self.device_overrides.integrated_gpus.iter().cloned().collect()
        });
        
        integrated_set.contains(&device_key)
    }
}

/// Global static function to check if device should be treated as integrated
pub fn is_integrated_gpu(vendor_id: &str, device_id: &str) -> bool {
    let overrides = GPU_OVERRIDES.get_or_init(|| GPUDeviceOverrides::load());
    overrides.is_integrated_device(vendor_id, device_id)
}
