//! Hardware sizing for the open model catalog.
//!
//! Listing a 70B model beside a 1B one with no guidance is not a choice, so the
//! catalog asks this module how much memory the machine can realistically give
//! a model and labels every entry against that figure.
//!
//! Detection reads total/available RAM and dedicated GPU memory once per
//! process. Nothing is written to disk, logged, or sent anywhere; the numbers
//! are only served on the loopback catalog endpoint so the UI can size entries.

use std::sync::OnceLock;

/// Memory a dedicated GPU needs kept back for the driver, display and context.
const GPU_RESERVE_MB: u64 = 1024;
/// Memory the OS and this app need kept back on a CPU-only budget.
const RAM_RESERVE_MB: u64 = 2048;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hardware {
    pub total_ram_mb: u64,
    pub available_ram_mb: u64,
    pub vram_mb: Option<u64>,
    pub gpu: Option<String>,
}

impl Hardware {
    /// How large a model this machine can hold before it starts swapping or
    /// spilling layers to the CPU.
    ///
    /// With a dedicated GPU the ceiling is GPU memory minus driver overhead;
    /// without one it is a slice of system RAM, because CPU inference still has
    /// to leave room for everything else that is running.
    pub fn budget_mb(&self) -> u64 {
        if self.total_ram_mb < 512 {
            return 0;
        }
        let ram_budget = self
            .total_ram_mb
            .saturating_sub(RAM_RESERVE_MB)
            .saturating_mul(55)
            / 100;
        match self.vram_mb {
            Some(vram) if vram >= 2048 => {
                let vram_budget = vram.saturating_sub(GPU_RESERVE_MB).saturating_mul(90) / 100;
                vram_budget.min(ram_budget.max(1024))
            }
            _ => ram_budget,
        }
    }

    pub fn detected(&self) -> bool {
        self.total_ram_mb > 0
    }

    pub fn summary(&self) -> String {
        if !self.detected() {
            return "Hardware size unknown — models are listed without a fit estimate.".to_string();
        }
        let gb = |mb: u64| mb as f64 / 1024.0;
        let gpu = match (&self.gpu, self.vram_mb) {
            (Some(name), Some(v)) => format!("{:.0} GB VRAM on {}", gb(v), name),
            (Some(name), None) => format!("shared graphics ({})", name),
            (None, Some(v)) => format!("{:.0} GB VRAM", gb(v)),
            (None, None) => "shared graphics".to_string(),
        };
        format!(
            "{:.0} GB RAM · {} · about {:.1} GB usable for a model",
            gb(self.total_ram_mb),
            gpu,
            gb(self.budget_mb())
        )
    }

    pub fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "detected": self.detected(),
            "total_ram_mb": self.total_ram_mb,
            "available_ram_mb": self.available_ram_mb,
            "vram_mb": self.vram_mb,
            "gpu": self.gpu,
            "budget_mb": self.budget_mb(),
            "summary": self.summary(),
        })
    }
}

/// Detected once, on first use, and reused for the life of the process.
pub fn detect() -> &'static Hardware {
    static DETECTED: OnceLock<Hardware> = OnceLock::new();
    DETECTED.get_or_init(|| {
        let (total_ram_mb, available_ram_mb) = platform::ram_mb();
        let (vram_mb, gpu) = platform::gpu();
        Hardware {
            total_ram_mb,
            available_ram_mb,
            vram_mb,
            gpu,
        }
    })
}

#[cfg(windows)]
mod platform {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    const HKEY_LOCAL_MACHINE: usize = 0x8000_0002;
    const KEY_READ: u32 = 0x0002_0019;
    const REG_SZ: u32 = 1;
    const REG_QWORD: u32 = 11;
    /// The display-adapter class key; its numbered subkeys each describe a GPU.
    const DISPLAY_CLASS: &str =
        r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}";

    #[repr(C)]
    struct MemoryStatus {
        length: u32,
        memory_load: u32,
        total_phys: u64,
        avail_phys: u64,
        total_page_file: u64,
        avail_page_file: u64,
        total_virtual: u64,
        avail_virtual: u64,
        avail_extended_virtual: u64,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GlobalMemoryStatusEx(status: *mut MemoryStatus) -> i32;
    }

    #[link(name = "advapi32")]
    extern "system" {
        fn RegOpenKeyExW(
            root: usize,
            sub_key: *const u16,
            options: u32,
            access: u32,
            out: *mut usize,
        ) -> i32;
        fn RegQueryValueExW(
            key: usize,
            name: *const u16,
            reserved: *mut u32,
            kind: *mut u32,
            data: *mut u8,
            len: *mut u32,
        ) -> i32;
        fn RegCloseKey(key: usize) -> i32;
    }

    fn wide(value: &str) -> Vec<u16> {
        OsStr::new(value)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    fn open(path: &str) -> Option<usize> {
        let name = wide(path);
        let mut key: usize = 0;
        let rc = unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, name.as_ptr(), 0, KEY_READ, &mut key) };
        (rc == 0 && key != 0).then_some(key)
    }

    fn query_u64(key: usize, name: &str) -> Option<u64> {
        let value = wide(name);
        let mut kind = 0u32;
        let mut data = [0u8; 8];
        let mut len = data.len() as u32;
        let rc = unsafe {
            RegQueryValueExW(
                key,
                value.as_ptr(),
                std::ptr::null_mut(),
                &mut kind,
                data.as_mut_ptr(),
                &mut len,
            )
        };
        (rc == 0 && kind == REG_QWORD && len >= 8).then(|| u64::from_le_bytes(data))
    }

    fn query_string(key: usize, name: &str) -> Option<String> {
        let value = wide(name);
        let mut kind = 0u32;
        let mut buffer = vec![0u8; 512];
        let mut len = buffer.len() as u32;
        let rc = unsafe {
            RegQueryValueExW(
                key,
                value.as_ptr(),
                std::ptr::null_mut(),
                &mut kind,
                buffer.as_mut_ptr(),
                &mut len,
            )
        };
        if rc != 0 || kind != REG_SZ || len < 2 {
            return None;
        }
        let units: Vec<u16> = buffer[..len as usize]
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .take_while(|unit| *unit != 0)
            .collect();
        if units.is_empty() {
            return None;
        }
        Some(String::from_utf16_lossy(&units))
    }

    pub fn ram_mb() -> (u64, u64) {
        let mut status = MemoryStatus {
            length: std::mem::size_of::<MemoryStatus>() as u32,
            memory_load: 0,
            total_phys: 0,
            avail_phys: 0,
            total_page_file: 0,
            avail_page_file: 0,
            total_virtual: 0,
            avail_virtual: 0,
            avail_extended_virtual: 0,
        };
        if unsafe { GlobalMemoryStatusEx(&mut status) } == 0 {
            return (0, 0);
        }
        const MB: u64 = 1024 * 1024;
        (status.total_phys / MB, status.avail_phys / MB)
    }

    /// Dedicated VRAM from the adapter's registry data. This covers NVIDIA, AMD
    /// and Intel, and unlike WMI's `AdapterRAM` it is not capped at 4 GB.
    pub fn gpu() -> (Option<u64>, Option<String>) {
        let mut best: Option<(u64, Option<String>)> = None;
        for index in 0..8 {
            let path = format!("{}\\{:04}", DISPLAY_CLASS, index);
            let Some(key) = open(&path) else { continue };
            let vram = query_u64(key, "HardwareInformation.qwMemorySize");
            let name = query_string(key, "DriverDesc");
            unsafe { RegCloseKey(key) };
            let Some(bytes) = vram else { continue };
            let mb = bytes / (1024 * 1024);
            if mb >= 512 && best.as_ref().map_or(true, |(seen, _)| mb > *seen) {
                best = Some((mb, name));
            }
        }
        match best {
            Some((mb, name)) => (Some(mb), name),
            None => nvidia_smi(),
        }
    }

    fn nvidia_smi() -> (Option<u64>, Option<String>) {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;

        let output = std::process::Command::new("nvidia-smi")
            .args([
                "--query-gpu=memory.total,name",
                "--format=csv,noheader,nounits",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        let Ok(output) = output else {
            return (None, None);
        };
        if !output.status.success() {
            return (None, None);
        }
        let text = String::from_utf8_lossy(&output.stdout);
        let mut best: Option<(u64, Option<String>)> = None;
        for line in text.lines() {
            let mut fields = line.split(',').map(str::trim);
            let Some(mb) = fields.next().and_then(|v| v.parse::<u64>().ok()) else {
                continue;
            };
            let name = fields.next().map(str::to_string);
            if best.as_ref().map_or(true, |(seen, _)| mb > *seen) {
                best = Some((mb, name));
            }
        }
        best.map(|(mb, name)| (Some(mb), name))
            .unwrap_or((None, None))
    }
}

#[cfg(not(windows))]
mod platform {
    pub fn ram_mb() -> (u64, u64) {
        #[cfg(target_os = "linux")]
        {
            if let Ok(text) = std::fs::read_to_string("/proc/meminfo") {
                let read = |key: &str| {
                    text.lines()
                        .find(|line| line.starts_with(key))
                        .and_then(|line| line.split_whitespace().nth(1))
                        .and_then(|kb| kb.parse::<u64>().ok())
                        .map(|kb| kb / 1024)
                };
                return (read("MemTotal:").unwrap_or(0), read("MemAvailable:").unwrap_or(0));
            }
        }
        (0, 0)
    }

    pub fn gpu() -> (Option<u64>, Option<String>) {
        (None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn machine(total_ram_mb: u64, vram_mb: Option<u64>) -> Hardware {
        Hardware {
            total_ram_mb,
            available_ram_mb: total_ram_mb / 2,
            vram_mb,
            gpu: None,
        }
    }

    #[test]
    fn a_gpu_budget_leaves_driver_headroom() {
        // 8 GB card: (8192 - 1024) * 0.9, below the 16 GB RAM slice.
        assert_eq!(machine(16384, Some(8192)).budget_mb(), 6451);
    }

    #[test]
    fn a_shared_graphics_budget_uses_system_ram() {
        assert_eq!(machine(16384, None).budget_mb(), 7884);
        assert_eq!(machine(32768, None).budget_mb(), 16896);
    }

    #[test]
    fn a_huge_card_still_respects_system_ram() {
        // A 24 GB card on an 8 GB machine cannot hold 24 GB of weights.
        let budget = machine(8192, Some(24576)).budget_mb();
        assert_eq!(budget, 3379, "RAM stays the limit: (8192 - 2048) * 0.55");
    }

    #[test]
    fn an_undetected_machine_has_no_budget() {
        assert_eq!(machine(0, None).budget_mb(), 0);
        assert!(!machine(0, None).detected());
        assert!(machine(0, None).summary().contains("unknown"));
    }

    #[test]
    fn the_summary_names_the_gpu_when_it_knows_it() {
        let hw = Hardware {
            total_ram_mb: 32768,
            available_ram_mb: 16384,
            vram_mb: Some(8192),
            gpu: Some("NVIDIA GeForce RTX 4060".to_string()),
        };
        let summary = hw.summary();
        assert!(summary.contains("32 GB RAM"), "{summary}");
        assert!(summary.contains("8 GB VRAM on NVIDIA GeForce RTX 4060"), "{summary}");
        assert!(summary.contains("about 6.3 GB usable"), "{summary}");
        assert_eq!(hw.json()["vram_mb"], 8192);
    }
}
