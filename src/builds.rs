use futures::StreamExt;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs,
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
};
use tracing::info;

use crate::{
    ui::LogItem,
    web::{self, ChampionsMap, FetchError},
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildSection {
    pub index: i64,
    pub id: String,
    pub version: String,
    pub official_version: String,
    pub pick_count: i64,
    pub win_rate: String,
    pub timestamp: i64,
    pub alias: String,
    pub name: String,
    pub position: String,
    pub skills: Option<Vec<String>>,
    pub spells: Option<Vec<String>>,
    pub item_builds: Vec<ItemBuild>,
    pub runes: Vec<Rune>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemBuild {
    pub title: String,
    pub associated_maps: Vec<i64>,
    pub associated_champions: Vec<i64>,
    pub blocks: Vec<Block>,
    pub map: String,
    pub mode: String,
    pub preferred_item_slots: Option<Vec<Value>>,
    pub sortrank: i64,
    pub started_from: String,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    #[serde(rename = "type")]
    pub type_field: String,
    pub items: Option<Vec<Item>>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: String,
    pub count: u8,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rune {
    pub alias: String,
    pub name: String,
    pub position: String,
    pub pick_count: u64,
    pub win_rate: String,
    pub primary_style_id: u64,
    pub sub_style_id: u64,
    pub selected_perk_ids: Vec<u64>,
    pub score: Option<f64>,
    #[serde(rename = "type", default = "empty_rune_type")]
    pub type_field: String,
}

pub fn empty_rune_type() -> String {
    String::new()
}

pub async fn fetch_and_apply(
    dir: &String,
    source: &String,
    champion: &String,
) -> Result<(), FetchError> {
    let sections = match web::fetch_build_file(source, champion, true).await {
        Ok(s) => s,
        Err(_) => {
            return Err(FetchError::Failed);
        }
    };

    let parent_dir = format!("{dir}/{champion}/Recommended");
    let _result = fs::create_dir_all(&parent_dir);

    let source_name = source.replace('.', "_");
    for (idx, b) in sections.iter().enumerate() {
        // let alias = &b.alias;
        let pos = &b.position;
        for (iidx, item) in b.item_builds.iter().enumerate() {
            let full_path =
                format!("{parent_dir}/{source_name}_{champion}_{pos}_{idx}_{iidx}.json");
            let mut f = fs::File::create(&full_path).unwrap();

            let buf = serde_json::to_string_pretty(&item).unwrap();
            f.write_all(buf[..].as_bytes()).unwrap();

            info!("builds saved to: {}", &full_path);
        }
    }

    Ok(())
}

pub async fn batch_apply(
    selected_sources: Vec<String>,
    champions_map: ChampionsMap,
    dir: String,
    is_tencent: bool,
    logs: Arc<Mutex<Vec<LogItem>>>,
) -> Result<(), ()> {
    let mut tasks = vec![];

    let folder = if is_tencent {
        format!("{dir}/Game/Config/Champions")
    } else {
        format!("{dir}/Config/Champions")
    };
    if Path::new(&folder).exists() {
        let _ = fs::remove_dir_all(&folder);
    } else {
        let _ = fs::create_dir_all(&folder);
    }

    for (champion, _) in champions_map.iter() {
        for source in selected_sources.iter() {
            let source = source.clone();
            let logs = logs.clone();
            let config_folder = folder.clone();

            let task = async move {
                info!("[apply_builds] started {:?} {:?}", &source, &champion);
                let r = fetch_and_apply(&config_folder, &source, champion).await;
                if r.is_ok() {
                    let mut logs = logs.lock().unwrap();
                    logs.push((source.clone(), champion.clone()));
                    drop(logs);
                } else {
                    info!("[apply_builds] failed {:?} {:?}", &source, &champion);
                }
            };
            tasks.push(task);
        }
    }

    futures::stream::iter(tasks)
        .buffer_unordered(10)
        .collect::<Vec<()>>()
        .await;

    Ok(())
}

#[derive(Default, Debug, Clone)]
pub struct BuildData(pub Vec<Rune>, pub Vec<ItemBuild>);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn apply_builds() -> Result<(), FetchError> {
        let target = String::from(".test");
        fetch_and_apply(&target, &String::from("op.gg"), &String::from("Rengar")).await
    }
}
