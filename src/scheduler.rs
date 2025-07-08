use crate::reminder::{load_reminders, Reminder};
use chrono::{Utc, Local};
use notify_rust::Notification;
use std::time::Duration;
use tokio::time::sleep;
use tokio_cron_scheduler::{JobScheduler, Job};
use cron::Schedule;
use std::str::FromStr;

pub async fn run_daemon() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting notification daemon...");
    
    let sched = JobScheduler::new().await?;
    
    // 启动调度器
    sched.start().await?;
    
    // 定期检查提醒数据并更新任务
    let job = Job::new_async("0/30 * * * * *", |_uuid, _l| {
        Box::pin(async move {
            update_scheduled_reminders().await.unwrap_or_else(|e| {
                eprintln!("Error updating reminders: {}", e);
            });
        })
    })?;
    
    sched.add(job).await?;
    
    println!("Notification daemon started. Press Ctrl+C to stop.");
    
    // 保持运行
    loop {
        sleep(Duration::from_secs(60)).await;
    }
}

async fn update_scheduled_reminders() -> Result<(), Box<dyn std::error::Error>> {
    let data = load_reminders()?;
    
    for reminder in &data.reminders {
        if reminder.completed || !reminder.notify_enabled {
            continue;
        }
        
        // 检查一次性到期提醒
        if let Some(due_at) = reminder.due_at {
            let now = Utc::now();
            let time_diff = due_at.signed_duration_since(now);
            
            // 如果在接下来的 5 分钟内到期，发送通知
            if time_diff.num_seconds() > 0 && time_diff.num_seconds() <= 300 {
                send_notification(reminder, "Due soon").await?;
            }
        }
        
        // 检查 cron 调度提醒
        if let Some(cron_expr) = &reminder.cron_schedule {
            if should_trigger_cron_reminder(cron_expr).await? {
                send_notification(reminder, "Scheduled reminder").await?;
            }
        }
    }
    
    Ok(())
}

async fn should_trigger_cron_reminder(cron_expr: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let schedule = Schedule::from_str(cron_expr)?;
    let now = Utc::now();
    
    // 获取下一次执行时间
    if let Some(next_time) = schedule.upcoming(Utc).next() {
        let time_diff = next_time.signed_duration_since(now);
        // 如果在接下来的 1 分钟内应该触发
        return Ok(time_diff.num_seconds() > 0 && time_diff.num_seconds() <= 60);
    }
    
    Ok(false)
}

async fn send_notification(reminder: &Reminder, notification_type: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 根据优先级在标题中显示不同的标识
    let priority_icon = match reminder.priority {
        crate::reminder::Priority::High => "🔴",
        crate::reminder::Priority::Medium => "🟡", 
        crate::reminder::Priority::Low => "🟢",
    };
    
    let title = format!("{} Reminder #{}: {}", priority_icon, reminder.id, notification_type);
    let body = format!("{}\nPriority: {}", reminder.message, reminder.priority);
    
    let mut notification = Notification::new();
    notification
        .summary(&title)
        .body(&body)
        .icon("dialog-information")
        .timeout(5000); // 5 seconds
    
    notification.show()?;
    
    println!("[{}] Sent notification for reminder #{}: {}", 
             Local::now().format("%Y-%m-%d %H:%M:%S"), 
             reminder.id, 
             reminder.message);
    
    Ok(())
}

// 手动测试通知功能
pub async fn test_notification() -> Result<(), Box<dyn std::error::Error>> {
    println!("Sending test notification...");
    
    Notification::new()
        .summary("Tools-RS Test Notification")
        .body("This is a test notification from your reminder tool!")
        .icon("dialog-information")
        .timeout(5000)
        .show()?;
    
    println!("Test notification sent!");
    Ok(())
}