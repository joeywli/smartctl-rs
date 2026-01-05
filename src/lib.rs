pub mod error;
pub mod device;
pub mod scan;

use std::{io, process::Output};

pub trait SmartCtlRunner {
    fn run(&self, args: &[&str]) -> io::Result<Output>;
}

pub struct RealSmartCtlRunner;

impl SmartCtlRunner for RealSmartCtlRunner {
    fn run(&self, args: &[&str]) -> io::Result<Output> {
        std::process::Command::new("smartctl")
            .args(args)
            .output()
    }
}