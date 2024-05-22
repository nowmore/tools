#![allow(dead_code)]
use crate::db::db;
use rusqlite::{params, Rows};
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};
fn kill(name: &str) {
    exec("taskkill", ["/f", "/t", "/im", name]);
}

pub fn kill_aria2c() {
    kill("aria2c.exe");
}

fn get_filename_from_path(path: &str) -> &str {
    match path.rsplit_once('/') {
        Some((_, filename)) => filename,
        None => path,
    }
}

fn is_proc_running(name: &str) -> bool {
    let output = Command::new("tasklist")
        .output()
        .expect("Failed to execute command");

    let output_str = String::from_utf8_lossy(&output.stdout);
    output_str.lines().any(|line| line.contains(name))
}

pub fn exec<I, S>(name: &str, args: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    Command::new(name)
        .args(args)
        .creation_flags(0x00000008)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
}

fn run_by_row(rows: &mut Rows) {
    while let Some(row) = rows.next().unwrap() {
        let cmd: String = row.get(0).unwrap();
        let args: String = row.get(1).unwrap();
        let args: Vec<_> = args.split_whitespace().collect();
        if !is_proc_running(get_filename_from_path(cmd.as_str())) {
            exec(cmd.as_str(), args);
        }
    }
}
pub fn run_proc(name: &str) {
    let ctx = db().lock().unwrap();
    let mut stmt = ctx
        .prepare("select cmd, args from procs WHERE name = ?1")
        .unwrap();
    let mut rows = stmt.query(params!(name)).unwrap();
    run_by_row(&mut rows);
}

pub fn run_procs() {
    let ctx = db().lock().unwrap();
    let mut stmt = ctx.prepare("select cmd, args from procs").unwrap();
    let mut rows = stmt.query(()).unwrap();
    run_by_row(&mut rows);
}
