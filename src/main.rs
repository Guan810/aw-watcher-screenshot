mod capture;
mod config;
mod event;

use anyhow::Result;
use capture::Capture;
use config::Config;
use std::path::Path;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    // 加载配置
    let config = Config::load_from("config/config.toml")?;

    // 确保保存目录存在
    let save_path = Path::new(&config.storage.local.path);
    std::fs::create_dir_all(save_path)?;

    // 创建统一捕获管理器
    let mut capture = Capture::new(config.monitors, None);

    // 创建通道接收截图结果
    let (tx, mut rx) = mpsc::channel(100);

    // 启动所有截图任务
    let task_count = capture.start_capture(tx);
    println!("启动了 {} 个截图任务", task_count);

    // 处理截图结果，截屏20次后退出
    let save_path = config.storage.local.path.clone();
    let handle = tokio::spawn(async move {
        let mut count = 0;
        while let Some(result) = rx.recv().await {
            count += 1;
            println!(
                "[{}/20] 收到截图: {} at {}",
                count, result.monitor_id, result.timestamp
            );

            // 保存图片
            let filename = format!(
                "{}_{}.png",
                result.monitor_id,
                result.timestamp.format("%Y%m%d_%H%M%S")
            );
            let filepath = Path::new(&save_path).join(filename);

            if let Err(e) = result.image.save(&filepath) {
                eprintln!("保存图片失败: {}", e);
            } else {
                println!("  -> 已保存到: {}", filepath.display());
            }

            if count >= 20 {
                println!("已完成20次截图，准备退出");
                break;
            }
        }
    });

    // 等待截图任务完成
    handle.await?;

    // 优雅关闭
    capture.shutdown().await;
    println!("程序已退出");

    Ok(())
}
