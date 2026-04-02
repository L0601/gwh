use chrono::{Local, NaiveDate};
use csv::{ReaderBuilder, WriterBuilder};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;

#[derive(Debug, Serialize)]
pub struct Contact {
    pub id: i64,
    pub name: String,
    pub remark: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct TaskItem {
    pub id: i64,
    pub title: String,
    pub contact_id: i64,
    pub contact_name: String,
    pub received_date: String,
    pub completed_date: Option<String>,
    pub fee: Option<f64>,
    pub print_fee: Option<f64>,
    pub settled_amount: Option<f64>,
    pub is_settled: bool,
    pub is_completed: bool,
    pub page_count: Option<i64>,
    pub word_count: Option<i64>,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct StatsSummary {
    pub task_count: i64,
    pub settled_total: f64,
}

#[derive(Debug, Serialize)]
pub struct StatsPayload {
    pub summary: StatsSummary,
    pub tasks: Vec<TaskItem>,
}

#[derive(Debug, Deserialize)]
pub struct ContactPayload {
    pub name: String,
    pub remark: String,
}

#[derive(Debug, Deserialize)]
pub struct TaskPayload {
    pub id: Option<i64>,
    pub title: String,
    pub contact_id: Option<i64>,
    pub received_date: String,
    pub completed_date: String,
    pub fee: String,
    pub print_fee: String,
    pub settled_amount: String,
    pub is_settled: bool,
    pub is_completed: bool,
    pub page_count: String,
    pub word_count: String,
}

#[derive(Debug, Deserialize)]
pub struct TaskFilters {
    pub status: String,
    pub contact_id: String,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Deserialize)]
pub struct StatsFilters {
    pub contact_id: String,
    pub start_date: String,
    pub end_date: String,
}

impl StatsFilters {
    pub fn with_defaults(self, start: NaiveDate, end: NaiveDate) -> Self {
        Self {
            contact_id: self.contact_id,
            start_date: if self.start_date.is_empty() {
                start.to_string()
            } else {
                self.start_date
            },
            end_date: if self.end_date.is_empty() {
                end.to_string()
            } else {
                self.end_date
            },
        }
    }
}

pub struct AppState {
    conn: Mutex<Connection>,
}

impl AppState {
    pub fn new(path: &Path) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        let state = Self {
            conn: Mutex::new(conn),
        };
        state.init()?;
        Ok(state)
    }

    pub fn list_contacts(&self) -> rusqlite::Result<Vec<Contact>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "select id, name, remark, created_at from contacts order by created_at desc, id desc",
        )?;
        let rows = stmt.query_map([], map_contact)?;
        rows.collect()
    }

    pub fn create_contact(&self, payload: ContactPayload) -> rusqlite::Result<Contact> {
        let conn = self.conn.lock().unwrap();
        let now = current_time_text();
        conn.execute(
            "insert into contacts(name, remark, created_at) values (?1, ?2, ?3)",
            params![payload.name.trim(), blank_to_none(&payload.remark), now],
        )?;
        let id = conn.last_insert_rowid();
        conn.query_row(
            "select id, name, remark, created_at from contacts where id = ?1",
            [id],
            map_contact,
        )
    }

    pub fn delete_contact(&self, id: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        let task_count: i64 = conn.query_row(
            "select count(1) from editing_tasks where contact_id = ?1",
            [id],
            |row| row.get(0),
        )?;
        if task_count > 0 {
            return Err(rusqlite::Error::InvalidQuery);
        }
        conn.execute("delete from contacts where id = ?1", [id])?;
        Ok(())
    }

    pub fn delete_task(&self, id: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("delete from editing_tasks where id = ?1", [id])?;
        Ok(())
    }

    pub fn list_tasks(&self, filters: TaskFilters) -> rusqlite::Result<Vec<TaskItem>> {
        let conn = self.conn.lock().unwrap();
        let sql = format!(
            "select t.id, t.title, t.contact_id, c.name, t.received_date, t.completed_date,
                    t.fee, t.print_fee, t.settled_amount, t.is_settled, t.is_completed,
                    t.page_count, t.word_count
             from editing_tasks t
             join contacts c on c.id = t.contact_id
             where 1 = 1 {} {} {} {}
             order by t.received_date desc, t.id desc",
            status_clause(&filters.status),
            contact_clause(&filters.contact_id),
            start_clause(&filters.start_date),
            end_clause(&filters.end_date)
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], map_task)?;
        rows.collect()
    }

    pub fn save_task(&self, payload: TaskPayload) -> rusqlite::Result<TaskItem> {
        let contact_id = payload.contact_id.ok_or(rusqlite::Error::InvalidQuery)?;
        let is_completed = payload.is_completed || payload.is_settled;
        let task_id = {
            let conn = self.conn.lock().unwrap();
            if let Some(id) = payload.id {
                conn.execute(
                    "update editing_tasks
                     set title = ?1, contact_id = ?2, received_date = ?3, completed_date = ?4,
                         fee = ?5, print_fee = ?6, settled_amount = ?7, is_settled = ?8,
                         is_completed = ?9, page_count = ?10, word_count = ?11
                     where id = ?12",
                    params![
                        payload.title.trim(),
                        contact_id,
                        payload.received_date,
                        blank_to_none(&payload.completed_date),
                        parse_optional_f64(&payload.fee),
                        parse_optional_f64(&payload.print_fee),
                        parse_optional_f64(&payload.settled_amount),
                        payload.is_settled,
                        is_completed,
                        parse_optional_i64(&payload.page_count),
                        parse_optional_i64(&payload.word_count),
                        id
                    ],
                )?;
                id
            } else {
                conn.execute(
                    "insert into editing_tasks(
                        title, contact_id, received_date, completed_date, fee, print_fee,
                        settled_amount, is_settled, is_completed, page_count, word_count
                     ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                    params![
                        payload.title.trim(),
                        contact_id,
                        payload.received_date,
                        blank_to_none(&payload.completed_date),
                        parse_optional_f64(&payload.fee),
                        parse_optional_f64(&payload.print_fee),
                        parse_optional_f64(&payload.settled_amount),
                        payload.is_settled,
                        is_completed,
                        parse_optional_i64(&payload.page_count),
                        parse_optional_i64(&payload.word_count)
                    ],
                )?;
                conn.last_insert_rowid()
            }
        };
        self.get_task(task_id)
    }

    pub fn stats_tasks(&self, filters: StatsFilters) -> rusqlite::Result<StatsPayload> {
        let tasks = self.list_tasks(TaskFilters {
            status: String::new(),
            contact_id: filters.contact_id.clone(),
            start_date: filters.start_date.clone(),
            end_date: filters.end_date.clone(),
        })?;
        let summary = StatsSummary {
            task_count: tasks.len() as i64,
            settled_total: tasks.iter().filter_map(|task| task.settled_amount).sum(),
        };
        Ok(StatsPayload { summary, tasks })
    }

    pub fn import_tasks(&self, path: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let mut reader = ReaderBuilder::new().from_path(path)?;
        let mut count = 0usize;
        for (index, row) in reader.deserialize::<TaskImportRecord>().enumerate() {
            let row = row.map_err(|err| {
                format!("第 {} 行导入失败，请检查列格式是否正确：{}", index + 2, err)
            })?;
            self.save_task(TaskPayload {
                id: None,
                title: row.title,
                contact_id: Some(self.ensure_contact(&row.contact_name)?),
                received_date: row.received_date,
                completed_date: row.completed_date.unwrap_or_default(),
                fee: row.fee.map(|v| v.to_string()).unwrap_or_default(),
                print_fee: row.print_fee.map(|v| v.to_string()).unwrap_or_default(),
                settled_amount: row.settled_amount.map(|v| v.to_string()).unwrap_or_default(),
                is_settled: row.is_settled,
                is_completed: row.is_completed,
                page_count: row.page_count.map(|v| v.to_string()).unwrap_or_default(),
                word_count: row.word_count.map(|v| v.to_string()).unwrap_or_default(),
            })
            .map_err(|err| format!("第 {} 行导入失败：{}", index + 2, err))?;
            count += 1;
        }
        Ok(count)
    }

    pub fn export_tasks(&self, path: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let tasks = self.list_tasks(TaskFilters {
            status: String::new(),
            contact_id: String::new(),
            start_date: String::new(),
            end_date: String::new(),
        })?;
        let mut writer = WriterBuilder::new().from_path(path)?;
        for task in &tasks {
            writer.serialize(TaskImportRecord {
                title: task.title.clone(),
                contact_name: task.contact_name.clone(),
                received_date: task.received_date.clone(),
                completed_date: task.completed_date.clone(),
                fee: task.fee,
                print_fee: task.print_fee,
                settled_amount: task.settled_amount,
                is_settled: task.is_settled,
                is_completed: task.is_completed,
                page_count: task.page_count,
                word_count: task.word_count,
            })?;
        }
        writer.flush()?;
        Ok(tasks.len())
    }

    pub fn export_template(&self, path: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let mut writer = WriterBuilder::new().from_path(path)?;
        writer.serialize(TaskImportRecord {
            title: "示例书稿".into(),
            contact_name: "示例对接人".into(),
            received_date: "2026-04-01".into(),
            completed_date: Some("2026-04-08".into()),
            fee: Some(300.0),
            print_fee: Some(20.0),
            settled_amount: Some(320.0),
            is_settled: true,
            is_completed: true,
            page_count: Some(120),
            word_count: Some(80000),
        })?;
        writer.flush()?;
        Ok(1)
    }

    fn init(&self) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "create table if not exists contacts (
                id integer primary key autoincrement,
                name text not null unique,
                remark text,
                created_at text not null
             );
             create table if not exists editing_tasks (
                id integer primary key autoincrement,
                title text not null,
                contact_id integer not null,
                received_date text not null,
                completed_date text,
                fee real,
                print_fee real,
                settled_amount real,
                is_settled integer not null default 0,
                is_completed integer not null default 0,
                page_count integer,
                word_count integer,
                foreign key(contact_id) references contacts(id)
             );",
        )?;
        ensure_column(&conn, "editing_tasks", "print_fee", "real")?;
        Ok(())
    }

    fn get_task(&self, id: i64) -> rusqlite::Result<TaskItem> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "select t.id, t.title, t.contact_id, c.name, t.received_date, t.completed_date,
                    t.fee, t.print_fee, t.settled_amount, t.is_settled, t.is_completed,
                    t.page_count, t.word_count
             from editing_tasks t
             join contacts c on c.id = t.contact_id
             where t.id = ?1",
            [id],
            map_task,
        )
    }

    fn ensure_contact(&self, name: &str) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().unwrap();
        if let Some(id) = conn
            .query_row("select id from contacts where name = ?1", [name], |row| row.get(0))
            .optional()?
        {
            return Ok(id);
        }
        let now = current_time_text();
        conn.execute(
            "insert into contacts(name, remark, created_at) values (?1, null, ?2)",
            params![name, now],
        )?;
        Ok(conn.last_insert_rowid())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TaskImportRecord {
    title: String,
    contact_name: String,
    received_date: String,
    completed_date: Option<String>,
    fee: Option<f64>,
    print_fee: Option<f64>,
    settled_amount: Option<f64>,
    is_settled: bool,
    is_completed: bool,
    page_count: Option<i64>,
    word_count: Option<i64>,
}

fn map_contact(row: &rusqlite::Row<'_>) -> rusqlite::Result<Contact> {
    Ok(Contact {
        id: row.get(0)?,
        name: row.get(1)?,
        remark: row.get(2)?,
        created_at: row.get(3)?,
    })
}

fn map_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<TaskItem> {
    let is_settled: bool = row.get(9)?;
    let is_completed: bool = row.get(10)?;
    Ok(TaskItem {
        id: row.get(0)?,
        title: row.get(1)?,
        contact_id: row.get(2)?,
        contact_name: row.get(3)?,
        received_date: row.get(4)?,
        completed_date: row.get(5)?,
        fee: row.get(6)?,
        print_fee: row.get(7)?,
        settled_amount: row.get(8)?,
        is_settled,
        is_completed,
        page_count: row.get(11)?,
        word_count: row.get(12)?,
        status: task_status(is_completed, is_settled).to_string(),
    })
}

fn task_status(is_completed: bool, is_settled: bool) -> &'static str {
    if is_settled {
        "SETTLED"
    } else if is_completed {
        "COMPLETED_UNSETTLED"
    } else {
        "IN_PROGRESS"
    }
}

fn status_clause(status: &str) -> String {
    match status {
        "IN_PROGRESS" => " and t.is_completed = 0 and t.is_settled = 0".into(),
        "COMPLETED_UNSETTLED" => " and t.is_completed = 1 and t.is_settled = 0".into(),
        "SETTLED" => " and t.is_settled = 1".into(),
        _ => String::new(),
    }
}

fn contact_clause(contact_id: &str) -> String {
    if contact_id.is_empty() {
        String::new()
    } else {
        format!(" and t.contact_id = {}", contact_id)
    }
}

fn start_clause(start_date: &str) -> String {
    if start_date.is_empty() {
        String::new()
    } else {
        format!(" and t.received_date >= '{}'", start_date)
    }
}

fn end_clause(end_date: &str) -> String {
    if end_date.is_empty() {
        String::new()
    } else {
        format!(" and t.received_date <= '{}'", end_date)
    }
}

fn blank_to_none(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn parse_optional_f64(value: &str) -> Option<f64> {
    value.trim().parse().ok()
}

fn parse_optional_i64(value: &str) -> Option<i64> {
    value.trim().parse().ok()
}

fn current_time_text() -> String {
    Local::now().format("%Y-%m-%d %H:%M").to_string()
}

fn ensure_column(conn: &Connection, table: &str, column: &str, definition: &str) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare(&format!("pragma table_info({})", table))?;
    let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;
    let exists = columns.filter_map(Result::ok).any(|name| name == column);
    if !exists {
        conn.execute(
            &format!("alter table {} add column {} {}", table, column, definition),
            [],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn memory_state() -> AppState {
        let conn = Connection::open_in_memory().unwrap();
        let state = AppState {
            conn: Mutex::new(conn),
        };
        state.init().unwrap();
        state
    }

    #[test]
    fn should_force_complete_when_task_settled() {
        let state = memory_state();
        let contact = state
            .create_contact(ContactPayload {
                name: "出版社A".into(),
                remark: String::new(),
            })
            .unwrap();

        let task = state
            .save_task(TaskPayload {
                id: None,
                title: "书稿A".into(),
                contact_id: Some(contact.id),
                received_date: "2026-04-01".into(),
                completed_date: String::new(),
                fee: "100".into(),
                print_fee: String::new(),
                settled_amount: "100".into(),
                is_settled: true,
                is_completed: false,
                page_count: String::new(),
                word_count: String::new(),
            })
            .unwrap();

        assert!(task.is_completed);
        assert_eq!(task.status, "SETTLED");
    }

    #[test]
    fn should_sum_settled_amount_in_stats() {
        let state = memory_state();
        let contact = state
            .create_contact(ContactPayload {
                name: "出版社B".into(),
                remark: String::new(),
            })
            .unwrap();

        for (title, settled) in [("书稿B1", "88"), ("书稿B2", "120")] {
            state
                .save_task(TaskPayload {
                    id: None,
                    title: title.into(),
                    contact_id: Some(contact.id),
                    received_date: "2026-04-01".into(),
                    completed_date: "2026-04-02".into(),
                    fee: settled.into(),
                    print_fee: String::new(),
                    settled_amount: settled.into(),
                    is_settled: true,
                    is_completed: true,
                    page_count: String::new(),
                    word_count: String::new(),
                })
                .unwrap();
        }

        let stats = state
            .stats_tasks(StatsFilters {
                contact_id: contact.id.to_string(),
                start_date: "2026-04-01".into(),
                end_date: "2026-04-30".into(),
            })
            .unwrap();

        assert_eq!(stats.summary.task_count, 2);
        assert_eq!(stats.summary.settled_total, 208.0);
    }

    #[test]
    fn should_reject_delete_contact_when_tasks_exist() {
        let state = memory_state();
        let contact = state
            .create_contact(ContactPayload {
                name: "出版社C".into(),
                remark: String::new(),
            })
            .unwrap();

        state
            .save_task(TaskPayload {
                id: None,
                title: "书稿C".into(),
                contact_id: Some(contact.id),
                received_date: "2026-04-01".into(),
                completed_date: String::new(),
                fee: String::new(),
                print_fee: String::new(),
                settled_amount: String::new(),
                is_settled: false,
                is_completed: false,
                page_count: String::new(),
                word_count: String::new(),
            })
            .unwrap();

        assert!(state.delete_contact(contact.id).is_err());
    }

    #[test]
    fn should_delete_task() {
        let state = memory_state();
        let contact = state
            .create_contact(ContactPayload {
                name: "出版社D".into(),
                remark: String::new(),
            })
            .unwrap();

        let task = state
            .save_task(TaskPayload {
                id: None,
                title: "书稿D".into(),
                contact_id: Some(contact.id),
                received_date: "2026-04-01".into(),
                completed_date: String::new(),
                fee: "50".into(),
                print_fee: String::new(),
                settled_amount: String::new(),
                is_settled: false,
                is_completed: false,
                page_count: String::new(),
                word_count: String::new(),
            })
            .unwrap();

        state.delete_task(task.id).unwrap();

        let tasks = state
            .list_tasks(TaskFilters {
                status: String::new(),
                contact_id: String::new(),
                start_date: String::new(),
                end_date: String::new(),
            })
            .unwrap();

        assert!(tasks.is_empty());
    }
}
