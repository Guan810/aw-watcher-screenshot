# Window Capture 使用文档

## 概述

`window.rs` 实现了对当前焦点窗口的智能截图功能,与 monitor 截图功能并行运行。

## 核心功能

### SafeWindow

单窗口截图封装,专注于捕获当前焦点窗口:

```rust
pub struct SafeWindow {
    last_capture_time: Option<DateTime<Utc>>,
    last_capture_dhash: Option<u64>,
    last_window_info: Option<WindowInfo>,
}
```

**主要方法:**
- `capture_once()`: 执行一次截图,支持基于时间和图像相似度的去重
- `last_window_info()`: 获取上次捕获的窗口信息(应用名和标题)

**去重策略:**
1. 时间间隔检查: 与上次截图的时间差必须 >= `enforce_interval`
2. 窗口ID检查: 检查是否是同一个窗口
3. 图像相似度检查: 使用 dHash 计算相似度,汉明距离 < `dhash_threshold` 视为相似

**特殊处理:**
- 自动跳过最小化窗口
- 没有焦点窗口时返回 `Ok(None)` 而非错误
- 窗口切换时强制捕获,即使图像相似

### Capture 管理器

统一管理 monitor 和 window 截图任务:

```rust
pub struct Capture {
    monitor_configs: HashMap<String, MonitorConfig>,
    window_config: Option<WindowConfig>,
    cancellation_token: CancellationToken,
    task_handles: Option<Vec<JoinHandle<()>>>,
}
```

## 使用示例

### 基础用法

```rust
use tokio::sync::mpsc;
use aw_watcher_screenshot::{Capture, Config};

#[tokio::main]
async fn main() -> Result<()> {
    // 加载配置
    let config = Config::load_from("config.toml")?;
    
    // 创建统一捕获管理器
    let mut capture = Capture::new(
        config.monitors,
        Some(config.window),
    );
    
    // 创建通道接收截图结果
    let (tx, mut rx) = mpsc::channel(100);
    
    // 启动所有截图任务(monitors + window)
    let task_count = capture.start_capture(tx);
    println!("启动了 {} 个截图任务", task_count);
    
    // 处理截图结果
    tokio::spawn(async move {
        while let Some(result) = rx.recv().await {
            println!("收到截图: {} at {}", result.monitor_id, result.timestamp);
            
            // 区分来源
            if result.monitor_id.starts_with("window_") {
                println!("  -> 这是窗口截图");
            } else {
                println!("  -> 这是显示器截图");
            }
            
            // 保存、上传等处理...
        }
    });
    
    // 运行一段时间
    tokio::time::sleep(Duration::from_secs(60)).await;
    
    // 优雅关闭
    capture.shutdown().await;
    
    Ok(())
}
```

### 配置示例

```toml
# config.toml

# 显示器截图配置(可配置多个)
[monitors.DP-1_1920_1080_0_0]
enable = true
interval = 1000              # 检查间隔 1秒
enforce_interval = 30000     # 最小截图间隔 30秒
dhash_resolution = 16
dhash_threshold = 10

# 窗口截图配置(全局唯一)
[window]
enable = true
interval = 1000              # 检查间隔 1秒
enforce_interval = 30000     # 最小截图间隔 30秒
dhash_resolution = 16
dhash_threshold = 10
enable_ocr = false           # OCR功能(未实现)
```

### 只使用窗口截图

```rust
// 不使用 monitor 截图,只使用 window 截图
let mut capture = Capture::new(
    HashMap::new(),           // 空的 monitor 配置
    Some(window_config),
);
```

### 信号处理

```rust
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    let mut capture = Capture::new(monitors, Some(window_config));
    let (tx, rx) = mpsc::channel(100);
    
    capture.start_capture(tx);
    
    // 等待 Ctrl+C
    signal::ctrl_c().await?;
    println!("收到关闭信号,正在停止...");
    
    // 优雅关闭所有任务
    let closed = capture.shutdown().await;
    println!("关闭了 {} 个任务", closed);
    
    Ok(())
}
```

## 截图结果格式

### Monitor 截图

```
monitor_id: "DP-1_1920_1080_0_0"
```

### Window 截图

```
monitor_id: "window_{app_name}_{window_id}"
例如: "window_firefox_12345"
```

## 错误处理

### 可恢复错误

- **No focused window found**: 没有焦点窗口时自动跳过,不计入错误次数
- **Window minimized**: 最小化窗口无法截图,自动跳过
- **时钟回退**: 记录警告但继续执行

### 不可恢复错误

- 连续错误超过 10 次: 任务自动终止
- 通道发送失败: 接收端已关闭

## OCR 功能预留

代码中已预留 OCR 集成位置:

```rust
// src/capture/window.rs 第 109 行
// TODO: OCR 功能预留位置
// if enable_ocr {
//     let ocr_result = perform_ocr(&image)?;
//     // 将 OCR 结果附加到 CaptureResult 或事件元数据中
// }
```

未来可以在此处添加:
1. 调用 OCR 引擎识别窗口文本
2. 将识别结果添加到 `CaptureResult` 的元数据中
3. 用于搜索、索引等功能

## 性能考虑

1. **独立任务**: 每个 monitor 和 window 都是独立的异步任务,互不阻塞
2. **无锁设计**: 每个任务独占自己的 `SafeMonitor`/`SafeWindow`,无锁竞争
3. **智能去重**: 避免重复截图相似的画面
4. **优雅关闭**: 使用 `CancellationToken` 实现快速响应关闭信号

## 调试技巧

### 查看捕获的窗口信息

```rust
if let Some((app, title)) = window.last_window_info() {
    println!("上次捕获: {} - {}", app, title);
}
```

### 日志级别

```toml
[logging]
level = "debug"  # 显示详细的窗口切换和截图信息
```

输出示例:
```
DEBUG Captured window: firefox - Mozilla Firefox
DEBUG No focused window, skipping capture
DEBUG Captured window: code - Visual Studio Code
```

## 限制和注意事项

1. **平台支持**: 依赖 `xcap` 库,支持 Linux(X11)、macOS、Windows
2. **Wayland 限制**: Linux Wayland 下功能受限
3. **窗口权限**: 某些系统窗口可能无法截图(如锁屏界面)
4. **性能**: 频繁的窗口切换会增加截图频率,注意 `enforce_interval` 设置

## 常见问题

### Q: 为什么没有捕获到窗口?

A: 检查:
1. `window.enable` 是否为 `true`
2. 是否有焦点窗口(有些情况下没有窗口获得焦点)
3. 窗口是否最小化
4. 查看日志中是否有错误信息

### Q: 截图频率太高怎么办?

A: 增加 `enforce_interval` 值,例如从 30000 增加到 60000(60秒)

### Q: 如何区分 monitor 和 window 截图?

A: 通过 `monitor_id` 字段:
- 以 `window_` 开头的是窗口截图
- 其他格式的是显示器截图

### Q: OCR 什么时候可用?

A: OCR 功能已预留接口,但尚未实现。可以通过实现 `perform_ocr()` 函数来集成 OCR 引擎(如 tesseract-rs)。
