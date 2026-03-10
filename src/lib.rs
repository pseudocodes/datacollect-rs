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
