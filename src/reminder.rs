use clap::Subcommand;
use chrono::{DateTime, Utc, Local, TimeZone};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum ReminderAction {
    #[command(about = "Add a new reminder")]
    Add {
        #[arg(help = "Reminder message")]
        message: String,
        #[arg(short, long, help = "Due date and time (e.g., '2024-12-31 15:30')")]
        due: Option<String>,
        #[arg(short, long, help = "Priority level (high, medium, low)", default_value = "medium")]
        priority: String,
        #[arg(short, long, help = "Cron schedule expression (e.g., '0 9 * * MON-FRI')")]
        cron: Option<String>,
        #[arg(short, long, help = "Enable system notifications", action = clap::ArgAction::SetTrue)]
        notify: bool,
    },
    #[command(about = "List all reminders")]
    List {
        #[arg(short, long, help = "Show only pending reminders")]
        pending: bool,
    },
    #[command(about = "Mark a reminder as completed")]
    Complete {
        #[arg(help = "Reminder ID")]
        id: usize,
    },
    #[command(about = "Remove a reminder")]
    Remove {
        #[arg(help = "Reminder ID")]
        id: usize,
    },
    #[command(about = "Test system notification")]
    TestNotify,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Reminder {
    pub id: usize,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub due_at: Option<DateTime<Utc>>,
    pub priority: Priority,
    pub completed: bool,
    pub cron_schedule: Option<String>,
    pub notify_enabled: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Priority {
    High,
    Medium,
    Low,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::High => write!(f, "HIGH"),
            Priority::Medium => write!(f, "MEDIUM"),
            Priority::Low => write!(f, "LOW"),
        }
    }
}

impl Priority {
    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "high" | "h" => Ok(Priority::High),
            "medium" | "med" | "m" => Ok(Priority::Medium),
            "low" | "l" => Ok(Priority::Low),
            _ => Err(format!("Invalid priority: {}. Use high, medium, or low", s)),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct RemindersData {
    pub reminders: Vec<Reminder>,
    pub next_id: usize,
}

fn get_data_file() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".tools-rs");
    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }
    path.push("reminders.json");
    path
}

pub fn load_reminders() -> Result<RemindersData, Box<dyn std::error::Error>> {
    let path = get_data_file();
    if !path.exists() {
        return Ok(RemindersData::default());
    }
    
    let content = fs::read_to_string(path)?;
    let data: RemindersData = serde_json::from_str(&content)?;
    Ok(data)
}

fn save_reminders(data: &RemindersData) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_data_file();
    let content = serde_json::to_string_pretty(data)?;
    fs::write(path, content)?;
    Ok(())
}

fn validate_cron_expression(cron_expr: &str) -> Result<(), Box<dyn std::error::Error>> {
    use cron::Schedule;
    use std::str::FromStr;
    
    Schedule::from_str(cron_expr)
        .map_err(|e| format!("Invalid cron expression '{}': {}", cron_expr, e))?;
    Ok(())
}

fn parse_datetime(date_str: &str) -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {
    let formats = [
        "%Y-%m-%d %H:%M",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d",
        "%m/%d/%Y %H:%M",
        "%m/%d/%Y",
    ];
    
    for format in &formats {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(date_str, format) {
            return Ok(Local.from_local_datetime(&dt).unwrap().with_timezone(&Utc));
        }
        if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, format) {
            let dt = date.and_hms_opt(0, 0, 0).unwrap();
            return Ok(Local.from_local_datetime(&dt).unwrap().with_timezone(&Utc));
        }
    }
    
    Err(format!("Unable to parse date: {}. Try format like '2024-12-31 15:30'", date_str).into())
}

pub async fn handle_reminder(action: ReminderAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ReminderAction::Add { message, due, priority, cron, notify } => {
            let mut data = load_reminders()?;
            
            let due_at = if let Some(due_str) = due {
                Some(parse_datetime(&due_str)?)
            } else {
                None
            };
            
            let priority = Priority::from_str(&priority)?;
            
            let cron_schedule = if let Some(cron_expr) = cron {
                validate_cron_expression(&cron_expr)?;
                Some(cron_expr)
            } else {
                None
            };
            
            let reminder = Reminder {
                id: data.next_id,
                message: message.clone(),
                created_at: Utc::now(),
                due_at,
                priority,
                completed: false,
                cron_schedule,
                notify_enabled: notify,
            };
            
            data.reminders.push(reminder);
            data.next_id += 1;
            
            save_reminders(&data)?;
            println!("Reminder added with ID: {}", data.next_id - 1);
            
            if notify {
                println!("System notifications enabled. Run 'tools-rs daemon' to start the notification service.");
            }
        }
        
        ReminderAction::List { pending } => {
            let data = load_reminders()?;
            let reminders: Vec<&Reminder> = if pending {
                data.reminders.iter().filter(|r| !r.completed).collect()
            } else {
                data.reminders.iter().collect()
            };
            
            if reminders.is_empty() {
                println!("No reminders found.");
                return Ok(());
            }
            
            println!("{:<3} {:<8} {:<6} {:<5} {:<19} {:<19} {:<15} {}", 
                     "ID", "STATUS", "PRI", "NOTIF", "CREATED", "DUE", "CRON", "MESSAGE");
            println!("{}", "─".repeat(100));
            
            for reminder in reminders {
                let status = if reminder.completed { "DONE" } else { "PENDING" };
                let notify_status = if reminder.notify_enabled { "ON" } else { "OFF" };
                let created = reminder.created_at.with_timezone(&Local).format("%Y-%m-%d %H:%M");
                let due = reminder.due_at
                    .map(|d| d.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "─".to_string());
                let cron_display = reminder.cron_schedule
                    .as_ref()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "─".to_string());
                
                println!("{:<3} {:<8} {:<6} {:<5} {:<19} {:<19} {:<15} {}", 
                         reminder.id, status, reminder.priority, notify_status, 
                         created, due, cron_display, reminder.message);
            }
        }
        
        ReminderAction::Complete { id } => {
            let mut data = load_reminders()?;
            
            if let Some(reminder) = data.reminders.iter_mut().find(|r| r.id == id) {
                reminder.completed = true;
                save_reminders(&data)?;
                println!("Reminder {} marked as completed.", id);
            } else {
                return Err(format!("Reminder with ID {} not found.", id).into());
            }
        }
        
        ReminderAction::Remove { id } => {
            let mut data = load_reminders()?;
            
            if let Some(pos) = data.reminders.iter().position(|r| r.id == id) {
                data.reminders.remove(pos);
                save_reminders(&data)?;
                println!("Reminder {} removed.", id);
            } else {
                return Err(format!("Reminder with ID {} not found.", id).into());
            }
        }
        
        ReminderAction::TestNotify => {
            crate::scheduler::test_notification().await?;
        }
    }
    
    Ok(())
}