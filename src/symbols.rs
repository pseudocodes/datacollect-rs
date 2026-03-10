/// DataCollect 动态库 C++ 导出符号名定义
///
/// 这些函数虽然在头文件中看似 C 函数，但实际以 C++ linkage 导出，
/// 因此必须使用 C++ mangled name 来加载。
///
/// 符号来源：
/// - Linux/macOS: 通过 `nm -gU` 从实际动态库提取
/// - Windows: 通过 `nm` 从 WinDataCollect.lib 提取

// ────────────────────────────────────────────────────
// CTP_GetSystemInfo(char*, int&) -> int
// 所有平台均导出
// ────────────────────────────────────────────────────

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub const GET_SYSTEM_INFO_SYMBOL: &[u8] = b"_Z17CTP_GetSystemInfoPcRi";

#[cfg(target_os = "windows")]
pub const GET_SYSTEM_INFO_SYMBOL: &[u8] = b"?CTP_GetSystemInfo@@YAHPEADAEAH@Z";

// ────────────────────────────────────────────────────
// CTP_GetDataCollectApiVersion(void) -> const char*
// macOS / Linux / Windows 均导出
// ────────────────────────────────────────────────────

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub const GET_API_VERSION_SYMBOL: &[u8] = b"_Z28CTP_GetDataCollectApiVersionv";

#[cfg(target_os = "windows")]
pub const GET_API_VERSION_SYMBOL: &[u8] = b"?CTP_GetDataCollectApiVersion@@YAPEBDXZ";

// ────────────────────────────────────────────────────
// CTP_GetSystemInfoUnAesEncode(char*, int&) -> int
// macOS / Linux 导出, Windows 未导出
// ────────────────────────────────────────────────────

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub const GET_SYSTEM_INFO_UN_AES_SYMBOL: &[u8] = b"_Z28CTP_GetSystemInfoUnAesEncodePcRi";

// ────────────────────────────────────────────────────
// CTP_GetRealSystemInfo(char*, int&) -> int
// 仅 Linux 导出 (头文件中未声明, 但动态库中存在)
// ────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
pub const GET_REAL_SYSTEM_INFO_SYMBOL: &[u8] = b"_Z21CTP_GetRealSystemInfoPcRi";
