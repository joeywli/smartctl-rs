use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::SmartCtlError;
use crate::{RealSmartCtlRunner, SmartCtlRunner};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartOutput {
    /// smartctl information
    pub smartctl: SmartCtl,

    /// Basic device information
    pub device: Option<DeviceInfo>,

    /// Model name
    pub model_name: Option<String>,

    /// Firmware version
    pub firmware_version: Option<String>,

    /// Serial number
    pub serial_number: Option<String>,

    /// Rotation rate
    pub rotation_rate: Option<u64>,

    /// User capacity
    pub user_capacity: Option<UserCapacity>,

    /// SMART Status
    pub smart_status: Option<SmartStatus>,

    /// ATA Specific Attributes
    pub ata_smart_attributes: Option<AtaSmartAttributes>,

    /// Power on time
    pub power_on_time: Option<PowerOnTime>,

    /// Power cycle count
    pub power_cycle_count: Option<u64>,

    /// Temperature information
    pub temperature: Option<Temperature>,

    /// NVMe SMART Health Information Log
    pub nvme_smart_health_information_log: Option<HashMap<String, i64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartCtl {
    #[serde(default)]
    pub messages: Vec<SmartCtlMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartCtlMessage {
    pub string: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub name: String,
    pub info_name: String,
    #[serde(rename = "type")]
    pub dev_type: String,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCapacity {
    pub blocks: u64,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartStatus {
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtaSmartAttributes {
    pub revision: u64,
    pub table: Vec<SmartAttribute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartAttribute {
    pub id: u64,
    pub name: String,
    pub value: u64,
    pub worst: u64,
    pub thresh: u64,
    pub when_failed: Option<String>,
    pub flags: SmartFlags,
    pub raw: SmartRawValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartFlags {
    pub value: u64,
    pub string: String,
    pub prefailure: bool,
    pub updated_online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartRawValue {
    pub value: u64,
    pub string: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerOnTime {
    pub hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Temperature {
    pub current: i64,
}

fn get_device_info_internal<R: SmartCtlRunner>(
    runner: &R,
    device_path: &str,
) -> Result<SmartOutput, SmartCtlError> {
    let args = ["--all", "--json", device_path];
    let output = runner.run(&args).map_err(|_| SmartCtlError::NotFound)?;

    // According to smartctl docs (https://linux.die.net/man/8/smartctl),
    // a non-zero exit code can indicate various conditions, we simply
    // parse the output if available anyway.
    if output.stdout.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SmartCtlError::CommandFailed(stderr.to_string()));
    }

    let parsed: SmartOutput = serde_json::from_slice(&output.stdout)?;

    if !parsed.smartctl.messages.is_empty() {
        for message in &parsed.smartctl.messages {
            if message.severity.to_lowercase() == "error" {
                return Err(SmartCtlError::CommandFailed(message.string.clone()));
            }
        }
    }

    Ok(parsed)
}

pub fn get_device_info(device_path: &str) -> Result<SmartOutput, SmartCtlError> {
    get_device_info_internal(&RealSmartCtlRunner, device_path)
}

#[cfg(test)]
mod tests {
    use std::{
        process::{ExitStatus, Output},
        vec,
    };

    #[cfg(unix)]
    use std::os::unix::process::ExitStatusExt;
    #[cfg(windows)]
    use std::os::windows::process::ExitStatusExt;

    use super::*;
    struct MockSmartCtlRunner {
        status_raw: i32,
        stdout_data: Vec<u8>,
        stderr_data: Vec<u8>,
    }

    impl SmartCtlRunner for MockSmartCtlRunner {
        fn run(&self, _args: &[&str]) -> std::io::Result<Output> {
            Ok(Output {
                status: ExitStatus::from_raw(self.status_raw),
                stdout: self.stdout_data.clone(),
                stderr: self.stderr_data.clone(),
            })
        }
    }

    #[test]
    fn test_success_hdd() {
        let json_data = include_bytes!("../tests/fixtures/hdd.json");
        let mock = MockSmartCtlRunner {
            status_raw: 0,
            stdout_data: json_data.to_vec(),
            stderr_data: vec![],
        };

        let result = get_device_info_internal(&mock, "/dev/sda");

        assert!(result.is_ok());
        let info = result.unwrap();

        // Test some key fields
        assert_eq!(info.device.as_ref().unwrap().name, "/dev/sda");
        assert_eq!(info.model_name.as_ref().unwrap(), "ST2000DM006-2DM164");
        assert_eq!(info.smart_status.as_ref().unwrap().passed, true);
        assert_eq!(info.user_capacity.as_ref().unwrap().bytes, 2000398934016);
        assert_eq!(info.temperature.as_ref().unwrap().current, 26);
    }

    #[test]
    fn test_success_sata_ssd() {
        let json_data = include_bytes!("../tests/fixtures/sata-ssd.json");
        let mock = MockSmartCtlRunner {
            status_raw: 0,
            stdout_data: json_data.to_vec(),
            stderr_data: vec![],
        };

        let result = get_device_info_internal(&mock, "/dev/sdb");

        assert!(result.is_ok());
        let info = result.unwrap();

        // Test some key fields
        assert_eq!(info.device.as_ref().unwrap().name, "/dev/sdb");
        assert_eq!(
            info.model_name.as_ref().unwrap(),
            "Samsung SSD 850 EVO M.2 250GB"
        );
        assert_eq!(info.smart_status.as_ref().unwrap().passed, true);
        assert_eq!(info.user_capacity.as_ref().unwrap().bytes, 250059350016);
        assert_eq!(info.temperature.as_ref().unwrap().current, 25);
    }

    #[test]
    fn test_success_nvme_ssd() {
        let json_data = include_bytes!("../tests/fixtures/nvme0.json");
        let mock = MockSmartCtlRunner {
            status_raw: 0,
            stdout_data: json_data.to_vec(),
            stderr_data: vec![],
        };

        let result = get_device_info_internal(&mock, "/dev/sdb");

        assert!(result.is_ok());
        let info = result.unwrap();

        // Test some key fields
        assert_eq!(info.device.as_ref().unwrap().name, "/dev/nvme0");
        assert_eq!(info.model_name.as_ref().unwrap(), "WDS100T1X0E-00AFY0");
        assert_eq!(info.smart_status.as_ref().unwrap().passed, true);
        assert_eq!(info.user_capacity.as_ref().unwrap().bytes, 1000204886016);
        assert_eq!(info.temperature.as_ref().unwrap().current, 37);
    }

    #[test]
    fn test_fail_message() {
        let json_data = include_bytes!("../tests/fixtures/perm.json");
        let mock = MockSmartCtlRunner {
            status_raw: 0,
            stdout_data: json_data.to_vec(),
            stderr_data: vec![],
        };

        let result = get_device_info_internal(&mock, "/dev/sdb");

        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            SmartCtlError::CommandFailed(_)
        ));
    }
}
