# datacollect-rs

CTP 看穿式终端信息采集模块的 Rust 封装。

通过 `libloading` 动态加载各平台（Linux / macOS / Windows）的 DataCollect 动态库，提供统一的 Rust 接口。

## 支持的平台和函数

| 函数 | macOS | Linux | Windows |
|------|:-----:|:-----:|:-------:|
| `get_system_info()` | ✅ | ✅ | ✅ |
| `get_system_info_un_aes()` | ✅ | ✅ | ❌ |
| `get_api_version()` | ✅ | ✅ | ✅ |
| `get_real_system_info()` | ❌ | ✅ | ❌ |

## 动态库文件

| 平台 | 库文件 |
|------|--------|
| Linux | `LinuxDataCollect.so` |
| macOS | `MacDataCollect.framework` |
| Windows | `WinDataCollect.dll` |

## 使用方式

```toml
[dependencies]
datacollect-rs = { path = "../datacollect-rs" }
```

```rust
use datacollect_rs::{DataCollectApi, resolve_datacollect_lib_path};

fn main() {
    // 自动解析平台对应的动态库路径
    let lib_path = resolve_datacollect_lib_path("/path/to/libs/");
    let api = DataCollectApi::new(&lib_path).expect("加载失败");

    // 获取版本号
    if let Some(ver) = api.get_api_version() {
        println!("DataCollect API: {}", ver);
    }

    // 采集终端信息
    let (data, len) = api.get_system_info().expect("采集失败");
    println!("采集到 {} 字节", len);
}
```

## 运行示例

```bash
cargo run --example basic -- /path/to/动态库目录/
```

## 参考

- 参照 [ctp-dyn](https://github.com/pseudocodes/ctp2rs/tree/master/ctp-dyn) 的 `libloading` 动态加载方式
- 仅依赖 `libloading`，无需 `bindgen` / `build.rs`
