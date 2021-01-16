#[macro_use]
extern crate rbatis;

use std::fs::OpenOptions;

use log::SetLoggerError;
use once_cell::sync::OnceCell;
use rbatis::rbatis::Rbatis;
use crate::db::delete_short_link;

mod db;

const DATABASE_FILE: &'static str = "data.db";

pub static RBATIS: OnceCell<Rbatis> = OnceCell::new();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup().await?;

    Ok(())
}

async fn setup_database() -> Result<(), rbatis::core::Error> {
    OpenOptions::new()
        .create_new(true)
        .open(DATABASE_FILE)
        .ok();

    let rb = RBATIS.get_or_init(Rbatis::new);
    rb
        .link(const_format::concatcp!("sqlite:", DATABASE_FILE) as &str)
        .await?;
    rb
        .exec("", include_str!("sql/create_table.sql"))
        .await?;

    Ok(())
}

fn setup_logger() -> Result<(), SetLoggerError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()
}

async fn setup() -> anyhow::Result<()> {
    setup_logger()?;
    setup_database().await?;

    Ok(())
}


#[tokio::test]
async fn test_insert() {
    use crate::db::{add_short_link, get_next_id, get_short_link};

    let add = |id| add_short_link(format!("http://example.com/test{}", id));

    setup().await.unwrap();

    let id = get_next_id().await.unwrap();
    let model = add(id).await.unwrap();

    // 测试重复添加同样的链接,是否返回同一个short
    assert_eq!(model.short.clone(), add(id).await.unwrap().short);

    // 测试获取link是否正常工作
    assert_eq!(
        get_short_link(model.short.clone())
            .await
            .unwrap()
            .unwrap()
            .id,
        model.id
    );

    // 不乱扔垃圾是好文明
    delete_short_link(model.short).await.unwrap();
}
