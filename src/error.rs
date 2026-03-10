use std::fmt;

/// DataCollect 操作结果
pub type Result<T> = std::result::Result<T, DataCollectError>;

/// DataCollect 错误类型
#[derive(Debug)]
pub enum DataCollectError {
    /// 动态库加载失败
    LibraryLoad(libloading::Error),
    /// 符号查找失败
    SymbolNotFound {
        symbol: String,
        source: libloading::Error,
    },
    /// 采集信息异常，包含错误位掩码
    CollectFailed(CollectErrorFlags),
}

impl fmt::Display for DataCollectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataCollectError::LibraryLoad(e) => {
                write!(f, "failed to load DataCollect library: {}", e)
            }
            DataCollectError::SymbolNotFound { symbol, source } => {
                write!(f, "symbol '{}' not found: {}", symbol, source)
            }
            DataCollectError::CollectFailed(flags) => write!(f, "{}", flags),
        }
    }
}

impl std::error::Error for DataCollectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DataCollectError::LibraryLoad(e) => Some(e),
            DataCollectError::SymbolNotFound { source, .. } => Some(source),
            DataCollectError::CollectFailed(_) => None,
        }
    }
}

/// 采集错误位掩码
///
/// `CTP_GetSystemInfo` 返回值为 `int`，非零时通过位掩码指示各采集项的状态。
/// 不同平台的 bit 覆盖范围不同：
/// - macOS: bit 0~7
/// - Linux: bit 0~8
/// - Windows: bit 0~9
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollectErrorFlags(pub i32);

impl CollectErrorFlags {
    /// 返回原始错误码
    #[inline]
    pub fn raw(&self) -> i32 {
        self.0
    }

    /// 终端类型未采集到 (bit 0)
    #[inline]
    pub fn terminal_type_missing(&self) -> bool {
        self.0 & (1 << 0) != 0
    }

    /// 信息采集时间获取异常 (bit 1)
    #[inline]
    pub fn time_error(&self) -> bool {
        self.0 & (1 << 1) != 0
    }

    /// IP 获取失败 (bit 2)
    #[inline]
    pub fn ip_failed(&self) -> bool {
        self.0 & (1 << 2) != 0
    }

    /// MAC 获取失败 (bit 3)
    #[inline]
    pub fn mac_failed(&self) -> bool {
        self.0 & (1 << 3) != 0
    }

    /// 设备名获取失败 (bit 4)
    #[inline]
    pub fn device_name_failed(&self) -> bool {
        self.0 & (1 << 4) != 0
    }

    /// 操作系统版本获取失败 (bit 5)
    #[inline]
    pub fn os_version_failed(&self) -> bool {
        self.0 & (1 << 5) != 0
    }

    /// 硬盘序列号获取失败 (bit 6)
    #[inline]
    pub fn disk_serial_failed(&self) -> bool {
        self.0 & (1 << 6) != 0
    }

    /// CPU序列号获取失败 (bit 7)
    #[inline]
    pub fn cpu_serial_failed(&self) -> bool {
        self.0 & (1 << 7) != 0
    }

    /// BIOS 获取失败 (bit 8, Linux/Windows only)
    #[inline]
    pub fn bios_failed(&self) -> bool {
        self.0 & (1 << 8) != 0
    }

    /// 系统盘分区信息获取失败 (bit 9, Windows only)
    #[inline]
    pub fn system_partition_failed(&self) -> bool {
        self.0 & (1 << 9) != 0
    }

    /// 返回所有失败项的描述列表
    pub fn failed_items(&self) -> Vec<&'static str> {
        let checks: &[(i32, &str)] = &[
            (1 << 0, "终端类型"),
            (1 << 1, "采集时间"),
            (1 << 2, "IP"),
            (1 << 3, "MAC"),
            (1 << 4, "设备名"),
            (1 << 5, "操作系统版本"),
            (1 << 6, "硬盘序列号"),
            (1 << 7, "CPU序列号"),
            (1 << 8, "BIOS"),
            (1 << 9, "系统盘分区"),
        ];
        checks
            .iter()
            .filter(|(mask, _)| self.0 & mask != 0)
            .map(|(_, desc)| *desc)
            .collect()
    }
}

impl fmt::Display for CollectErrorFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let items = self.failed_items();
        if items.is_empty() {
            write!(f, "CollectError(0x{:04x})", self.0)
        } else {
            write!(
                f,
                "CollectError(0x{:04x}): 以下采集项失败: {}",
                self.0,
                items.join(", ")
            )
        }
    }
}
