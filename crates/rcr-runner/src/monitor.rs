use std::time::Duration;
use sysinfo::{Pid, ProcessesToUpdate, System};
use tracing::trace;

/// Monitors a process's CPU and memory usage by periodically sampling.
pub struct ProcessMonitor {
    pid: u32,
    cpu_samples: Vec<f32>,
    mem_samples_kb: Vec<u64>,
    system: System,
}

impl ProcessMonitor {
    pub fn new(pid: u32) -> Self {
        Self {
            pid,
            cpu_samples: Vec::new(),
            mem_samples_kb: Vec::new(),
            system: System::new(),
        }
    }

    /// Take a single sample of the process's CPU % and memory usage.
    pub fn sample(&mut self) {
        self.system.refresh_processes(
            ProcessesToUpdate::Some(&[Pid::from_u32(self.pid)]),
            true,
        );

        let pid = Pid::from_u32(self.pid);
        if let Some(process) = self.system.process(pid) {
            let cpu = process.cpu_usage();
            let mem = process.memory() / 1024; // bytes -> KB

            self.cpu_samples.push(cpu);
            self.mem_samples_kb.push(mem);

            trace!(pid = self.pid, cpu_pct = cpu, mem_kb = mem, "Process sample");
        }
    }

    /// Run a sampling loop at the given interval. Call via tokio::spawn.
    pub async fn sample_loop(mut self, interval: Duration) {
        let mut interval = tokio::time::interval(interval);
        loop {
            interval.tick().await;
            self.sample();
        }
    }

    /// Get the peak CPU % observed across all samples.
    pub fn peak_cpu(&self) -> Option<f32> {
        self.cpu_samples.iter().cloned().reduce(f32::max)
    }

    /// Get the peak memory usage in KB across all samples.
    pub fn peak_mem_kb(&self) -> Option<i64> {
        self.mem_samples_kb.iter().cloned().max().map(|v| v as i64)
    }
}