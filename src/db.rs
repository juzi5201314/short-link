use rbatis::rbatis::Rbatis;
use serde_json::Value;

use crate::{RBATIS, DATABASE_FILE};
use rbatis::crud::CRUD;
use std::fs::OpenOptions;

#[crud_enable(table_name: link_data)]
#[derive(Debug, Clone)]
pub struct Model {
    pub id: Option<u32>,
    pub short: String,
    pub link: String,
}

#[cold]
pub async fn delete_short_link(short: String) -> anyhow::Result<()> {
    let db = get_db();

    db
        .remove_by_wrapper::<Model>("", &db
            .new_wrapper()
            .eq("short", &short)
            .check()?,
        )
        .await?;

    Ok(())
}

/// None: 短链接不存在
pub async fn get_short_link(short: String) -> anyhow::Result<Option<Model>> {
    let db = get_db();

    let model: Option<Model> = db
        .fetch_by_wrapper("", &db
            .new_wrapper()
            .eq("short", &short)
            .check()?,
        )
        .await?;

    if let Some(model) = model {
        Ok(Some(model))
    } else {
        Ok(None)
    }
}

pub async fn add_short_link(link: String, custom: Option<String>) -> anyhow::Result<Model> {
    let db = get_db();

    let get_short = || {
        async {
            let id = get_next_id().await.expect("无法获取下一个id");
            match custom.clone() {
                Some(s) => (s, id),
                _ => {
                    // 根据下一个自增id生成对应标识符
                    (base_62::encode(&id.to_be_bytes()), id)
                }
            }
        }
    };

    // 查询link是否重复
    let model: Option<Model> = db
        .fetch_by_wrapper("", &db
            .new_wrapper()
            .eq("link", &link)
            .check()?,
        )
        .await?;
    // 如果重复,返回已有的short
    if let Some(model) = model {
        return Ok(model);
    }

    let short = get_short().await.0;

    // 查询short是否重复
    // 在自定义短标识符的情况下可能会出现重复, 但概率很小.
    let model: Option<Model> = db
        .fetch_by_wrapper("", &db
            .new_wrapper()
            .eq("short", &short)
            .check()?,
        )
        .await?;
    // 如果重复,填入uuid占位. uuid重复的概率近乎于0.
    if let Some(_) = model {
        let uid = uuid::Uuid::new_v4().to_string();
        db.save("", &Model {
            id: None,
            short: uid.clone(),
            link: uid,
        }).await?;
    }

    // 重新获取id
    let (short, id) = get_short().await;

    let model = Model {
        id: None,
        short,
        link,
    };

    db.save("", &model).await?;

    Ok(Model {
        id: Some(id),
        ..model
    })
}

pub fn get_db() -> &'static Rbatis {
    RBATIS
        .get()
        .expect("数据库未初始化")
}

pub async fn get_next_id() -> anyhow::Result<u32> {
    if let Value::Array(arr) = get_db()
        .fetch("", "select seq from sqlite_sequence WHERE name = 'link_data'")
        .await? {
        if let Some(Value::Object(map)) = arr.first() {
            if let Some(Value::Number(num)) = map.get("seq") {
                if let Some(count) = num.as_u64() {
                    return Ok((count + 1) as u32);
                }
            }
        }
    }
    Err(anyhow::Error::msg("获取id失败"))
}

pub async fn setup_database() -> Result<(), rbatis::core::Error> {
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
