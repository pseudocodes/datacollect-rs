//! # datacollect-rs
//!
//! CTP 看穿式终端信息采集模块的 Rust 封装。
//!
//! 通过 `libloading` 动态加载各平台（Linux / macOS / Windows）的
//! DataCollect 动态库，提供统一的 Rust 接口。
//!
//! ## 快速上手
//!
//! ```no_run
//! use datacollect_rs::{DataCollectApi, resolve_datacollect_lib_path};
//!
//! // 自动解析平台对应的动态库路径
//! let lib_path = resolve_datacollect_lib_path("/path/to/libs/");
//! let api = DataCollectApi::new(&lib_path).expect("加载失败");
//!
//! // 获取版本号
//! if let Some(ver) = api.get_api_version() {
//!     println!("DataCollect API: {}", ver);
//! }
//!
//! // 采集终端信息（加密）
//! let (data, len) = api.get_system_info().expect("采集失败");
//! println!("采集到 {} 字节", len);
//! ```

#![allow(non_upper_case_globals)]

pub mod api;
pub mod error;
pub mod symbols;

pub use api::{resolve_datacollect_lib_path, DataCollectApi};
pub use error::{CollectErrorFlags, DataCollectError, Result};

/// 将 `Vec<u8>` 转换为 `[i8; N]` 定长数组。
///
/// 用于将采集到的终端信息填入 CTP API 要求的 `c_char` 定长字段
/// （如 `TThostFtdcClientSystemInfoType`）。
///
/// - 若 `src.len() < N`，剩余位置填充 `0`
/// - 若 `src.len() >= N`，截取前 `N` 字节
///
/// # 示例
/// ```
/// use datacollect_rs::vec_u8_to_i8_array;
///
/// let data = vec![0x48u8, 0x65, 0x6C, 0x6C, 0x6F];
/// let arr: [i8; 8] = vec_u8_to_i8_array(&data);
/// assert_eq!(&arr[..5], &[0x48, 0x65, 0x6C, 0x6C, 0x6F_i8]);
/// assert_eq!(&arr[5..], &[0, 0, 0]);
/// ```
pub fn vec_u8_to_i8_array<const N: usize>(src: &[u8]) -> [i8; N] {
    let mut dst = [0i8; N];
    let copy_len = src.len().min(N);
    // SAFETY: u8 和 i8 具有相同的大小和对齐，仅符号解释不同
    unsafe {
        std::ptr::copy_nonoverlapping(src.as_ptr() as *const i8, dst.as_mut_ptr(), copy_len);
    }
    dst
}
