use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;

use crate::capture::Capture;
use crate::config::Config;

#[derive(Parser)]
#[command(name = "aw-watcher-screenshot")]
#[command(about = "ActivityWatch 截图监控工具", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 列出所有可识别的显示器ID
    ListMonitors,

    /// 开始截图
    Capture {
        /// 配置文件路径
        #[arg(short, long, default_value = "config/config.toml")]
        config: PathBuf,

        /// 最大截图次数（0表示无限制）
        #[arg(short, long, default_value = "0")]
        max_count: usize,

        /// 日志等级 (trace, debug, info, warn, error)
        #[arg(short, long)]
        log_level: Option<String>,

        /// 本地存储路径（设置此项将自动启用本地存储）
        #[arg(short = 's', long)]
        storage_path: Option<PathBuf>,
    },
}

/// CLI 入口函数
pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::ListMonitors => {
            list_monitors()?;
        }
        Commands::Capture {
            config,
            max_count,
            log_level,
            storage_path,
        } => {
            start_capture(config, max_count, log_level, storage_path).await?;
        }
    }

    Ok(())
}

/// 列出所有可识别的显示器ID
fn list_monitors() -> Result<()> {
    println!("正在扫描显示器...\n");

    let monitor_ids = Capture::get_all_monitors_id()?;

    if monitor_ids.is_empty() {
        println!("未检测到任何显示器");
    } else {
        println!("检测到 {} 个显示器:\n", monitor_ids.len());
        for (idx, id) in monitor_ids.iter().enumerate() {
            println!("  [{}] {}", idx + 1, id);
        }
    }

    Ok(())
}

/// 开始截图任务
async fn start_capture(
    config_path: PathBuf,
    max_count: usize,
    log_level: Option<String>,
    storage_path: Option<PathBuf>,
) -> Result<()> {
    // 初始化日志
    let log_level = log_level.as_deref().unwrap_or("info");
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    // 加载配置
    let mut config = Config::load_from(&config_path)?;

    // 如果指定了存储路径，覆盖配置
    if let Some(path) = storage_path {
        config.storage.local.enable = true;
        config.storage.local.path = path.to_string_lossy().to_string();
        println!("已启用本地存储，路径: {}", config.storage.local.path);
    }

    // 确保保存目录存在
    let save_path = Path::new(&config.storage.local.path);
    std::fs::create_dir_all(save_path)?;

    // 创建统一捕获管理器
    let mut capture = Capture::new(config.monitors.clone(), None);

    // 创建通道接收截图结果
    let (tx, mut rx) = mpsc::channel(100);

    // 启动所有截图任务
    let task_count = capture.start_capture(tx);
    println!("启动了 {} 个截图任务", task_count);

    // 确定截图限制
    let count_limit = if max_count == 0 {
        usize::MAX
    } else {
        max_count
    };

    // 处理截图结果
    let save_path_clone = config.storage.local.path.clone();
    let handle = tokio::spawn(async move {
        let mut count = 0;
        while let Some(result) = rx.recv().await {
            count += 1;

            let progress = if count_limit == usize::MAX {
                format!("[{}]", count)
            } else {
                format!("[{}/{}]", count, count_limit)
            };

            println!(
                "{} 收到截图: {} at {}",
                progress, result.monitor_id, result.timestamp
            );

            // 保存图片
            let filename = format!(
                "{}_{}.png",
                result.monitor_id,
                result.timestamp.format("%Y%m%d_%H%M%S")
            );
            let filepath = Path::new(&save_path_clone).join(filename);

            if let Err(e) = result.image.save(&filepath) {
                eprintln!("保存图片失败: {}", e);
            } else {
                println!("  -> 已保存到: {}", filepath.display());
            }

            if count >= count_limit {
                println!("已完成 {} 次截图，准备退出", count_limit);
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
