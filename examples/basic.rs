//! 基本使用示例
//!
//! 运行方式: cargo run --example basic

use datacollect_rs::{resolve_datacollect_lib_path, DataCollectApi};
use std::path::Path;

/// 获取项目内 lib 目录下对应平台的动态库路径
fn get_lib_path() -> std::path::PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    #[cfg(target_os = "linux")]
    {
        Path::new(manifest_dir).join("lib/linux64/6.7.0_P1_CP_LinuxDataCollect/LinuxDataCollect.so")
    }

    #[cfg(target_os = "macos")]
    {
        // 默认使用测试密钥版本，如需生产密钥可改为 sfit_pro_1.0_20250325
        resolve_datacollect_lib_path(
            Path::new(manifest_dir).join("lib/macos/sfit_tst_1.0_20250325"),
        )
    }

    #[cfg(target_os = "windows")]
    {
        Path::new(manifest_dir).join("lib/win64/v6.6.7_20220304_clientdll64/WinDataCollect.dll")
    }
}

fn main() {
    let lib_path = get_lib_path();
    println!("动态库路径: {:?}", lib_path);

    // 加载动态库
    let api = match DataCollectApi::new(&lib_path) {
        Ok(api) => {
            println!("✅ 动态库加载成功");
            api
        }
        Err(e) => {
            eprintln!("❌ 加载失败: {}", e);
            std::process::exit(1);
        }
    };

    // 获取版本号
    match api.get_api_version() {
        Some(version) => println!("📋 API 版本: {}", version),
        None => println!("⚠️  该库未导出 CTP_GetDataCollectApiVersion"),
    }

    // 采集终端信息（加密）
    println!("\n--- 采集加密终端信息 ---");
    match api.get_system_info() {
        Ok((data, len)) => {
            println!("✅ 采集成功: {} 字节", len);
            let preview_len = std::cmp::min(len, 64);
            let hex: String = data[..preview_len]
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ");
            println!("   前 {} 字节: {}", preview_len, hex);
        }
        Err(e) => {
            eprintln!("❌ 采集失败: {}", e);
        }
    }

    // macOS / Linux: 采集未加密终端信息
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        println!("\n--- 采集未加密终端信息 ---");
        match api.get_system_info_un_aes() {
            Some(Ok((data, len))) => {
                println!("✅ 采集成功: {} 字节", len);
                match String::from_utf8(data.clone()) {
                    Ok(text) => println!("   内容: {}", text),
                    Err(_) => {
                        let hex: String = data
                            .iter()
                            .map(|b| format!("{:02x}", b))
                            .collect::<Vec<_>>()
                            .join(" ");
                        println!("   (非 UTF-8) hex: {}", hex);
                    }
                }
            }
            Some(Err(e)) => eprintln!("❌ 采集失败: {}", e),
            None => println!("⚠️  该库未导出 CTP_GetSystemInfoUnAesEncode"),
        }
    }

    // Linux only: 获取真实系统信息
    #[cfg(target_os = "linux")]
    {
        println!("\n--- 获取真实系统信息 ---");
        match api.get_real_system_info() {
            Some(Ok((data, len))) => {
                println!("✅ 采集成功: {} 字节", len);
                let hex: String = data
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                println!("   hex: {}", hex);
            }
            Some(Err(e)) => eprintln!("❌ 采集失败: {}", e),
            None => println!("⚠️  该库未导出 CTP_GetRealSystemInfo"),
        }
    }
}
