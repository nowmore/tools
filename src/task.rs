use crate::aria2;
use crate::bgminfo;
use crate::db;
use crate::log;
use crate::moe;
use crate::proc;
use crate::taskinfo;
use chrono::{prelude::*, Days};
use regex::Regex;
use tokio::{signal::ctrl_c, sync::mpsc};
use tracing::{debug, error, info};

fn generate_tasks() -> Result<(), Box<dyn std::error::Error>> {
    let bgms = bgminfo::get_new_bgms()?;
    let mut tasks: Vec<taskinfo::Task> = Vec::new();
    let days = Days::new(7);
    let now = Local::now();
    for mut bgm in bgms {
        let start_date = NaiveDate::parse_from_str(&bgm.start_date, "%Y%m%d");
        let weekday = Weekday::try_from(bgm.weekday - 1);
        if weekday.is_err() || start_date.is_err() {
            bgm.state = 2;
            continue;
        }

        let mut begin = start_date.unwrap();
        let wd = weekday.unwrap();
        let mut idx = 0;

        while idx < bgm.episode_count {
            tasks.push(taskinfo::Task {
                id: 0,
                bgm_id: bgm.id,
                episode: bgm.episode + idx,
                regex: format!(".*{}.*{:02}.*", bgm.regex, bgm.episode + idx),
                path: bgm.path.clone(),
                uri: "".to_string(),
                gid: "".to_string(),
                exec_time: Local
                    .from_local_datetime(&begin.and_hms_opt(bgm.clock as u32, 00, 00).unwrap())
                    .unwrap()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                create_time: now.format("%Y-%m-%d %H:%M:%S").to_string(),
                finish_time: "".to_string(),
                state: 0,
            });

            idx += 1;

            begin = if begin.weekday() != wd {
                while begin.weekday() != wd {
                    begin = begin.succ_opt().unwrap()
                }
                begin
            } else {
                begin.checked_add_days(days).unwrap()
            };
        }
        bgm.state = 1;
        bgminfo::update_bgm_state(bgm)?;
    }

    taskinfo::generate_tasks(&tasks)
}

async fn update_task_status(task: &mut taskinfo::Task) {
    let rs = aria2::status(&task.gid).await;
    match rs {
        Ok((s, c, t)) => {
            if s == "complete" || c == t {
                debug!("task:{} is completed!", task.id);
                task.state = 1;
                task.finish_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                let _ = aria2::remove(&task.gid);
            }
        }
        Err(e) => error!("get task:{} status error:{:?}", task.id, e),
    }
}
async fn exec_task(task: &mut taskinfo::Task, torrents: &Vec<moe::Torrent>) {
    if task.uri.len() == 0 && torrents.len() > 0 {
        let re = Regex::new(&task.regex);
        if re.is_err() {
            error!("invalid regex:{} of task:{}", task.regex, task.id);
            task.state = 4;
            return;
        }
        let re = re.unwrap();
        for t in torrents {
            if re.is_match(&t.title) {
                info!("task:{}, title:{}, {}", task.id, t.title, t.magnet);
                task.uri = t.magnet.clone();
                break;
            }
        }
    }

    if task.uri.len() != 0 {
        match aria2::download(&task.uri, &task.path, "").await {
            Ok(gid) => {
                task.state = 3;
                task.gid = gid;
            }
            Err(e) => error!("download task:{} error:{:?}", task.id, e),
        }
    }
}

async fn exec_tasks(
    tasks: &mut Vec<taskinfo::Task>,
    last: &mut NaiveDateTime,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut new_tasks = taskinfo::get_ready_tasks()?;
    tasks.append(&mut new_tasks);

    let init_state_tasks: Vec<_> = tasks
        .iter_mut()
        .filter(|t| t.state == 2 && t.uri == "")
        .collect();
    let now = Local::now().naive_local();
    let torrents: Vec<moe::Torrent> =
        if init_state_tasks.len() > 0 && now.signed_duration_since(*last).num_minutes() > 10 {
            let earliest_time = &init_state_tasks
                .iter()
                .min_by(|t1, t2| t1.exec_time.cmp(&t2.exec_time))
                .unwrap()
                .exec_time;
            let earliest =
                NaiveDateTime::parse_from_str(earliest_time.as_str(), "%Y-%m-%d %H:%M:%S").unwrap();

            match moe::get_torrents(&earliest).await {
                Err(e) => {
                    error!(
                        "get torrents failed, please check your proxy config! {:?}",
                        e
                    );
                    Vec::new()
                }
                Ok(result) => {
                    *last = now;
                    result
                }
            }
        } else {
            Vec::new()
        };

    for mut task in tasks.iter_mut() {
        match task.state {
            2 => exec_task(&mut task, &torrents).await,
            1 => task.finish_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            3 => update_task_status(&mut task).await,
            _ => (),
        }

        taskinfo::update_task(task)?;
    }

    tasks.retain_mut(|t| t.state != 4 && t.state != 1);
    Ok(())
}

pub async fn exec() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = log::init_log();
    db::init_db();
    proc::run_procs();
    generate_tasks()?;
    let (tx, mut rx) = mpsc::channel(1);

    tokio::spawn(async move {
        let notify = db::notify();
        let mut sub = notify.subscribe();
        loop {
            match sub.recv().await {
                Ok((action, database, tbl, _row_id)) => {
                    if action == rusqlite::hooks::Action::SQLITE_INSERT
                        && database == "main"
                        && tbl == "bgm"
                    {
                        generate_tasks().unwrap();
                        tx.send(1).await.unwrap();
                    }
                }
                Err(e) => error!("error while recv db notify: {:?}", e),
            }
        }
    });

    tokio::spawn(async move {
        let mut last = NaiveDateTime::UNIX_EPOCH;
        let mut secs: u64;
        let mut tasks: Vec<taskinfo::Task> = taskinfo::get_incomplete_tasks().unwrap();

        loop {
            exec_tasks(&mut tasks, &mut last).await.unwrap();
            secs = if tasks.len() > 0 {
                1
            } else {
                let next = taskinfo::get_next_exec_time().unwrap();
                info!("next active time: {next}");
                NaiveDateTime::parse_from_str(next.as_str(), "%Y-%m-%d %H:%M:%S")
                    .unwrap()
                    .signed_duration_since(Local::now().naive_local())
                    .num_seconds() as u64
            };
            let _ = tokio::time::timeout(std::time::Duration::from_secs(secs), rx.recv()).await;
        }
    });

    ctrl_c().await?;
    Ok(())
}
