use crate::db::db;

#[derive(Debug)]
pub struct Task {
    pub id: u32,
    pub bgm_id: u32,
    pub episode: u8,
    pub regex: String,
    pub path: String,
    pub uri: String,
    pub gid: String,
    pub exec_time: String,
    pub create_time: String,
    pub finish_time: String,
    pub state: u8,
}

pub fn generate_tasks(tasks: &Vec<Task>) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = db().lock()?;
    let mut stmt = ctx.prepare(
        "INSERT INTO task(bgm_id, episode, regex, path, exec_time, create_time) 
            VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
    )?;

    for task in tasks {
        stmt.execute(rusqlite::params![
            task.bgm_id,
            task.episode,
            task.regex,
            task.path,
            task.exec_time,
            task.create_time
        ])?;
    }
    Ok(())
}

pub fn get_ready_tasks() -> Result<Vec<Task>, Box<dyn std::error::Error>> {
    let ctx = db().lock()?;
    let mut stmt = ctx
        .prepare("SELECT id, bgm_id, episode, regex, path, exec_time FROM task WHERE state = 0 and exec_time <= datetime(CURRENT_TIMESTAMP, 'localtime')")
        ?;
    let tasks = stmt.query_map([], |row| {
        Ok(Task {
            id: row.get(0)?,
            bgm_id: row.get(1)?,
            episode: row.get(2)?,
            regex: row.get(3)?,
            path: row.get(4)?,
            uri: "".to_string(),
            gid: "".to_string(),
            exec_time: row.get(5)?,
            create_time: "".to_string(),
            finish_time: "".to_string(),
            state: 2,
        })
    })?;
    let mut result: Vec<Task> = Vec::new();

    for task in tasks {
        result.push(task?);
    }

    Ok(result)
}

pub fn update_task(task: &Task) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = db().lock()?;
    let mut stmt = ctx.prepare(
        "UPDATE task SET state = ?1, uri = ?2, gid = ?3, finish_time = ?4 WHERE id = ?5",
    )?;

    stmt.execute(rusqlite::params![
        task.state,
        task.uri,
        task.gid,
        task.finish_time,
        task.id
    ])?;
    Ok(())
}

pub fn get_incomplete_tasks() -> Result<Vec<Task>, Box<dyn std::error::Error>> {
    let ctx = db().lock()?;
    let mut stmt = ctx
        .prepare("SELECT id, bgm_id, episode, regex, path, uri, gid, state FROM task WHERE (state = 2 or state = 3) and datetime(CURRENT_TIMESTAMP, 'localtime')")
        ?;
    let tasks = stmt.query_map([], |row| {
        Ok(Task {
            id: row.get(0)?,
            bgm_id: row.get(1)?,
            episode: row.get(2)?,
            regex: row.get(3)?,
            path: row.get(4)?,
            uri: row.get(5).unwrap_or_default(),
            gid: row.get(6).unwrap_or_default(),
            exec_time: "".to_string(),
            create_time: "".to_string(),
            finish_time: "".to_string(),
            state: row.get(7).unwrap_or_default(),
        })
    })?;
    let mut result: Vec<Task> = Vec::new();
    for task in tasks {
        result.push(task?);
    }
    Ok(result)
}

pub fn get_next_exec_time() -> Result<String, Box<dyn std::error::Error>> {
    let ctx = db().lock()?;
    let mut stmt = ctx
        .prepare("SELECT IFNULL(MIN(exec_time), DATETIME(DATE('now','localtime'), '+1 day')) FROM task WHERE state = 0  AND exec_time BETWEEN DATETIME(DATE('now','localtime')) and DATETIME(DATE('now','localtime') || ' 23:59:59')")
        ?;
    if let Some(row) = stmt.query(())?.next()? {
        let time: String = row.get(0)?;
        return Ok(time);
    }
    Err("select ifnull return no data".into())
}
