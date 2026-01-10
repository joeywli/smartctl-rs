use serde::{Deserialize, Serialize};

use crate::{RealSmartCtlRunner, SmartCtlRunner, error::SmartCtlError};

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanDeviceList {
    pub devices: Vec<ScanDevice>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanDevice {
    pub name: String,
    pub info_name: String,
    #[serde(rename = "type")]
    pub dev_type: String,
    pub protocol: String,
}

fn scan_devices_internal<R: SmartCtlRunner>(runner: &R) -> Result<Vec<ScanDevice>, SmartCtlError> {
    let args = ["--scan", "--json"];
    let output = runner.run(&args).map_err(|_| SmartCtlError::NotFound)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SmartCtlError::CommandFailed(stderr.to_string()));
    }

    let parsed: ScanDeviceList = serde_json::from_slice(&output.stdout)?;
    Ok(parsed.devices)
}

pub fn scan_devices() -> Result<Vec<ScanDevice>, SmartCtlError> {
    scan_devices_internal(&RealSmartCtlRunner)
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
    fn test_success_scan() {
        let json_data = include_bytes!("../tests/fixtures/scan.json");
        let mock = MockSmartCtlRunner {
            status_raw: 0,
            stdout_data: json_data.to_vec(),
            stderr_data: vec![],
        };

        let result = scan_devices_internal(&mock);

        assert!(result.is_ok());
        let info = result.unwrap();

        assert_eq!(info.len(), 3);

        // Verify first device
        assert_eq!(info[0].name, "/dev/sda");
        assert_eq!(info[0].info_name, "/dev/sda");
        assert_eq!(info[0].dev_type, "scsi");
        assert_eq!(info[0].protocol, "SCSI");

        // Verify second device
        assert_eq!(info[1].name, "/dev/sdb");
        assert_eq!(info[1].info_name, "/dev/sdb");
        assert_eq!(info[1].dev_type, "scsi");
        assert_eq!(info[1].protocol, "SCSI");

        // Verify third device
        assert_eq!(info[2].name, "/dev/nvme0");
        assert_eq!(info[2].info_name, "/dev/nvme0");
        assert_eq!(info[2].dev_type, "nvme");
        assert_eq!(info[2].protocol, "NVMe");
    }

    #[test]
    fn test_error_scan() {
        let mock = MockSmartCtlRunner {
            status_raw: 1,
            stdout_data: vec![],
            stderr_data: vec![],
        };

        let result = scan_devices_internal(&mock);

        assert!(result.is_err());
    }
}
