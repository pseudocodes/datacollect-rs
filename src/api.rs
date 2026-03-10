use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::path::Path;

use libloading::Library;

use crate::error::*;
use crate::symbols::*;

/// 推荐缓冲区大小（头文件要求至少 270 字节，此处留有余量）
const DEFAULT_BUFFER_SIZE: usize = 512;

/// FFI 函数签名类型别名
///
/// C++ `int CTP_GetSystemInfo(char* pSystemInfo, int& nLen)`
/// 在 ABI 层面 `int&` 等价于 `int*`
type FnGetSystemInfo = unsafe extern "C" fn(*mut c_char, *mut c_int) -> c_int;

/// C++ `const char* CTP_GetDataCollectApiVersion(void)`
type FnGetApiVersion = unsafe extern "C" fn() -> *const c_char;

/// CTP 终端信息采集 API
///
/// 通过 `libloading` 动态加载 DataCollect 库，提供终端信息采集功能。
/// 内部存储已解析的函数指针，避免每次调用时重复查找符号。
///
/// # 平台差异
/// - **所有平台**: `get_system_info()`, `get_api_version()`
/// - **macOS + Linux**: `get_system_info_un_aes()`
/// - **仅 Linux**: `get_real_system_info()`
///
/// # 示例
/// ```no_run
/// use datacollect_rs::DataCollectApi;
///
/// let api = DataCollectApi::new("path/to/LinuxDataCollect.so").unwrap();
/// println!("版本: {:?}", api.get_api_version());
///
/// let (data, len) = api.get_system_info().unwrap();
/// println!("采集到 {} 字节数据", len);
/// ```
pub struct DataCollectApi {
    /// 持有动态库句柄，保证函数指针的生命周期
    /// 注意: _lib 必须在所有函数指针字段之后被 drop（Rust 按声明顺序 drop）
    /// 但由于我们存储的是原始函数指针（非 Symbol<'a>），这里不存在生命周期问题，
    /// 只需保证 _lib 在 DataCollectApi 存活期间不被卸载即可。
    _lib: Library,

    /// CTP_GetSystemInfo: 获取 AES+RSA 加密的终端信息
    fn_get_system_info: FnGetSystemInfo,

    /// CTP_GetDataCollectApiVersion: 获取 API 版本号（可选，部分旧版 Linux 库可能不导出）
    fn_get_api_version: Option<FnGetApiVersion>,

    /// CTP_GetSystemInfoUnAesEncode: 获取未 AES 加密的终端信息
    /// 仅 macOS / Linux 平台可用
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn_get_system_info_un_aes: Option<FnGetSystemInfo>,

    /// CTP_GetRealSystemInfo: 获取真实系统信息（未在头文件中声明，但库中导出）
    /// 仅 Linux 平台可用
    #[cfg(target_os = "linux")]
    fn_get_real_system_info: Option<FnGetSystemInfo>,
}

// DataCollectApi 内部函数指针指向已加载的动态库，不涉及共享可变状态
// Send: 可以安全地跨线程转移所有权
// Sync: 所有方法都是 &self（只读），函数指针本身只读调用，是线程安全的
//       底层 C++ 采集函数每次调用使用独立的栈上缓冲区，不涉及共享状态
unsafe impl Send for DataCollectApi {}
unsafe impl Sync for DataCollectApi {}

impl DataCollectApi {
    /// 加载 DataCollect 动态库并解析导出符号
    ///
    /// # 参数
    /// - `lib_path` - 动态库的文件路径
    ///   - Linux: `LinuxDataCollect.so`
    ///   - macOS: `MacDataCollect.framework/Versions/A/MacDataCollect`
    ///   - Windows: `WinDataCollect.dll`
    ///
    /// # 错误
    /// - `DataCollectError::LibraryLoad` - 动态库文件不存在或加载失败
    /// - `DataCollectError::SymbolNotFound` - 必要符号 `CTP_GetSystemInfo` 未找到
    pub fn new<P: AsRef<Path>>(lib_path: P) -> Result<Self> {
        let lib =
            unsafe { Library::new(lib_path.as_ref()).map_err(DataCollectError::LibraryLoad)? };

        // 必须的函数: CTP_GetSystemInfo
        let fn_get_system_info: FnGetSystemInfo = unsafe {
            let sym: libloading::Symbol<FnGetSystemInfo> = lib
                .get(GET_SYSTEM_INFO_SYMBOL)
                .map_err(|e| DataCollectError::SymbolNotFound {
                    symbol: String::from_utf8_lossy(GET_SYSTEM_INFO_SYMBOL).into_owned(),
                    source: e,
                })?;
            // 将 Symbol<'_, F> 解引用为原始函数指针，脱离 lifetime 约束
            *sym
        };

        // 可选函数: CTP_GetDataCollectApiVersion
        let fn_get_api_version: Option<FnGetApiVersion> = unsafe {
            lib.get::<FnGetApiVersion>(GET_API_VERSION_SYMBOL)
                .ok()
                .map(|sym| *sym)
        };

        // macOS / Linux: CTP_GetSystemInfoUnAesEncode
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        let fn_get_system_info_un_aes: Option<FnGetSystemInfo> = unsafe {
            lib.get::<FnGetSystemInfo>(GET_SYSTEM_INFO_UN_AES_SYMBOL)
                .ok()
                .map(|sym| *sym)
        };

        // Linux only: CTP_GetRealSystemInfo
        #[cfg(target_os = "linux")]
        let fn_get_real_system_info: Option<FnGetSystemInfo> = unsafe {
            lib.get::<FnGetSystemInfo>(GET_REAL_SYSTEM_INFO_SYMBOL)
                .ok()
                .map(|sym| *sym)
        };

        Ok(Self {
            _lib: lib,
            fn_get_system_info,
            fn_get_api_version,
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            fn_get_system_info_un_aes,
            #[cfg(target_os = "linux")]
            fn_get_real_system_info,
        })
    }

    /// 获取 AES+RSA 加密的终端信息
    ///
    /// 调用底层 `CTP_GetSystemInfo`，返回加密后的终端信息字节数据。
    /// 该数据用于 CTP 的 `ReqAuthenticate`（客户端认证）请求。
    ///
    /// # 返回
    /// - `Ok((data, length))` - `data` 为采集到的原始字节（可能包含 `\0`），
    ///   `length` 为有效数据长度
    /// - `Err(CollectFailed)` - 返回值非零，包含错误位掩码
    ///
    /// # 注意
    /// 返回的数据中可能包含 `\0`，请使用内存复制而非字符串操作处理。
    pub fn get_system_info(&self) -> Result<(Vec<u8>, usize)> {
        let mut buf = vec![0u8; DEFAULT_BUFFER_SIZE];
        let mut len: c_int = buf.len() as c_int;

        let ret = unsafe {
            (self.fn_get_system_info)(buf.as_mut_ptr() as *mut c_char, &mut len as *mut c_int)
        };

        if ret != 0 {
            return Err(DataCollectError::CollectFailed(CollectErrorFlags(ret)));
        }

        let actual_len = len as usize;
        buf.truncate(actual_len);
        Ok((buf, actual_len))
    }

    /// 获取未 AES 加密的终端信息
    ///
    /// 调用底层 `CTP_GetSystemInfoUnAesEncode`。
    /// 仅在 macOS / Linux 平台可用。
    ///
    /// # 返回
    /// - `Some(Ok((data, length)))` - 函数存在且调用成功
    /// - `Some(Err(CollectFailed))` - 函数存在但采集失败
    /// - `None` - 当前动态库未导出该函数
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub fn get_system_info_un_aes(&self) -> Option<Result<(Vec<u8>, usize)>> {
        let f = self.fn_get_system_info_un_aes?;

        let mut buf = vec![0u8; DEFAULT_BUFFER_SIZE];
        let mut len: c_int = buf.len() as c_int;

        let ret = unsafe { f(buf.as_mut_ptr() as *mut c_char, &mut len as *mut c_int) };

        if ret != 0 {
            return Some(Err(DataCollectError::CollectFailed(CollectErrorFlags(ret))));
        }

        let actual_len = len as usize;
        buf.truncate(actual_len);
        Some(Ok((buf, actual_len)))
    }

    /// 获取真实系统信息
    ///
    /// 调用底层 `CTP_GetRealSystemInfo`。
    /// 仅在 Linux 平台可用（该函数未在头文件中声明，但动态库中存在）。
    ///
    /// # 返回
    /// - `Some(Ok((data, length)))` - 函数存在且调用成功
    /// - `Some(Err(CollectFailed))` - 函数存在但采集失败
    /// - `None` - 当前动态库未导出该函数
    #[cfg(target_os = "linux")]
    pub fn get_real_system_info(&self) -> Option<Result<(Vec<u8>, usize)>> {
        let f = self.fn_get_real_system_info?;

        let mut buf = vec![0u8; DEFAULT_BUFFER_SIZE];
        let mut len: c_int = buf.len() as c_int;

        let ret = unsafe { f(buf.as_mut_ptr() as *mut c_char, &mut len as *mut c_int) };

        if ret != 0 {
            return Some(Err(DataCollectError::CollectFailed(CollectErrorFlags(ret))));
        }

        let actual_len = len as usize;
        buf.truncate(actual_len);
        Some(Ok((buf, actual_len)))
    }

    /// 获取 DataCollect API 版本号
    ///
    /// 调用底层 `CTP_GetDataCollectApiVersion`。
    /// 版本号格式: `Sfit + pro/tst + 秘钥版本 + 编译时间 + 版本(内部)`
    ///
    /// # 返回
    /// - `Some(version_string)` - 成功获取版本号
    /// - `None` - 当前动态库未导出该函数
    pub fn get_api_version(&self) -> Option<String> {
        let f = self.fn_get_api_version?;
        unsafe {
            let ptr = f();
            if ptr.is_null() {
                return None;
            }
            Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
        }
    }
}

/// 根据平台在指定目录下解析 DataCollect 动态库的完整路径
///
/// # 平台规则
/// - **Linux**: `<dir>/LinuxDataCollect.so`
/// - **macOS**: 优先查找 `MacDataCollect.framework/Versions/A/MacDataCollect`，
///   然后查找 `MacDataCollect.framework/MacDataCollect`
/// - **Windows**: `<dir>/WinDataCollect.dll`
///
/// # 示例
/// ```no_run
/// use datacollect_rs::resolve_datacollect_lib_path;
///
/// let path = resolve_datacollect_lib_path("/path/to/libs/");
/// println!("动态库路径: {:?}", path);
/// ```
pub fn resolve_datacollect_lib_path<P: AsRef<Path>>(dir: P) -> std::path::PathBuf {
    let dir = dir.as_ref();

    #[cfg(target_os = "linux")]
    {
        dir.join("LinuxDataCollect.so")
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: 优先检查 Versions/A/ 结构
        let versioned = dir.join("MacDataCollect.framework/Versions/A/MacDataCollect");
        if versioned.exists() {
            return versioned;
        }
        // 其次检查顶层 symlink
        dir.join("MacDataCollect.framework/MacDataCollect")
    }

    #[cfg(target_os = "windows")]
    {
        dir.join("WinDataCollect.dll")
    }
}
