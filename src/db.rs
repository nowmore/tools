use rusqlite::{hooks::Action, Connection};
use std::sync::{Mutex, OnceLock};
use tokio::sync::broadcast;
const DB_FILE: &str = "I:/programs/bangumi/bgm.db";

#[derive(Debug)]
pub struct Db {
    ctx: Mutex<Connection>,
    tx: broadcast::Sender<(Action, String, String, i64)>,
}

impl Db {
    fn new() -> Self {
        let ctx = Connection::open(DB_FILE).unwrap();
        let (tx, _) = broadcast::channel(1);
        Db {
            ctx: Mutex::new(ctx),
            tx,
        }
    }
}
static DB: OnceLock<Db> = OnceLock::new();

pub fn init_db() {
    DB.set(Db::new()).unwrap();
    DB.get().unwrap().ctx.lock().unwrap().update_hook(Some(
        |action, db: &str, tbl: &str, row_id| {
            DB.get()
                .unwrap()
                .tx
                .send((action, db.to_string(), tbl.to_string(), row_id))
                .unwrap();
        },
    ));
}

pub fn db() -> &'static Mutex<Connection> {
    &DB.get().unwrap().ctx
}

pub fn notify() -> &'static broadcast::Sender<(Action, String, String, i64)> {
    &DB.get().unwrap().tx
}
