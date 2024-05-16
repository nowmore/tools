const DEFAULT_PATH: &str = "D:/download";
use crate::db::db;
#[derive(Debug)]
pub struct Bgm {
    pub id: u32,
    pub name: String,
    pub chinese: String,
    pub start_date: String,
    pub weekday: u8,
    pub clock: u8,
    pub episode: u8,
    pub episode_count: u8,
    pub regex: String,
    pub path: String,
    pub state: u8,
}

pub fn get_new_bgms() -> Result<Vec<Bgm>, Box<dyn std::error::Error>> {
    let ctx = db().lock()?;
    let mut stmt = ctx.prepare("SELECT * FROM bgm WHERE state = ?")?;
    let bgms = stmt.query_map([0], |row| {
        Ok(Bgm {
            id: row.get(0)?,
            name: row.get(1)?,
            chinese: row.get(2).unwrap_or_default(),
            start_date: row.get(3).unwrap_or_default(),
            weekday: row.get(4)?,
            clock: row.get(5).unwrap_or_default(),
            episode: row.get(6).unwrap_or_default(),
            episode_count: row.get(7).unwrap_or_default(),
            regex: row.get(8)?,
            path: row.get(9).unwrap_or(DEFAULT_PATH.to_string()),
            state: row.get(10)?,
        })
    })?;

    let mut result: Vec<Bgm> = Vec::new();
    for bgm in bgms {
        result.push(bgm?);
    }
    Ok(result)
}

pub fn update_bgm_state(bgm: Bgm) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = db().lock()?;
    let mut stmt = ctx.prepare("UPDATE bgm SET state = ? WHERE id = ?")?;

    stmt.execute(rusqlite::params![bgm.state, bgm.id])?;
    Ok(())
}
