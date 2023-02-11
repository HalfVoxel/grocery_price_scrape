use std::{
    collections::{HashMap, HashSet},
    fs::{self, DirEntry},
    io::{self, Read},
    path::{Path, PathBuf},
    time::Instant,
};

use crate::{
    parse::{parse_item, ParsedItem, StoreData},
    types::ScrapedItem,
};
use chrono::{Datelike, NaiveDate, Weekday};
use flate2::read::GzDecoder;
use fs::File;
use fuse_rust::Fuse;
use indicatif::{ParallelProgressIterator, ProgressIterator};
use io::BufReader;
use lazy_static::lazy_static;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rocket::{
    fs::{FileServer, NamedFile},
    get, launch, post, routes,
    serde::json::Json,
    State,
};
use serde::Deserialize;
use serde::Serialize;
use ts_rs::TS;
use types::{ComparePrice, SoldInUnit};

mod parse;
mod types;

#[derive(TS, Serialize, Clone, Debug)]
#[ts(export)]
struct DataForDay {
    date: NaiveDate,
    store_id: u32,
    data: TrimmedParsedItem,
}

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
#[ts(export)]
pub struct TrimmedParsedItem {
    pub price: f32,
    pub compare: Option<ComparePrice>,
    pub sold_in_unit: SoldInUnit,
    pub unit_weight: Option<f32>,
}

#[derive(TS, Serialize, Clone, Debug)]
#[ts(export)]
struct Item {
    name: String,
    url: String,
    price_data: Vec<DataForDay>,
}

struct ServerState {
    items: Vec<Item>,
}

lazy_static! {
    static ref STATIC_PATH: PathBuf = std::fs::canonicalize("../frontend/public").unwrap();
    // static ref THUMBNAIL_PATH: PathBuf =
    //     std::fs::canonicalize("../home_scraper/output/parse/thumbnails").unwrap();
}

#[derive(Deserialize, Clone, Debug)]
struct SearchParams {
    name: String,
}

#[post("/search", data = "<data>")]
async fn search(state: &State<ServerState>, data: Json<SearchParams>) -> Json<Vec<&Item>> {
    let fuse = Fuse {
        location: 0,            // Approx where to start looking for the pattern
        distance: 100,          // Maximum distance the score should scale to
        threshold: 0.6,         // A threshold for guess work
        max_pattern_length: 32, // max valid pattern length
        is_case_sensitive: false,
        tokenize: false, // the input search text should be tokenized
    };

    let query = data.name.to_lowercase();
    let mut scored_items = state
        .items
        .iter()
        .filter_map(|item| {
            fuse.search_text_in_string(&query, &item.name)
                .map(|s| (s.score, item))
        })
        .collect::<Vec<_>>();

    scored_items.sort_by(|(score_a, _), (score_b, _)| score_a.partial_cmp(score_b).unwrap());

    println!(
        "{:?}",
        scored_items
            .iter()
            .take(10)
            .map(|(score, item)| (score + (item.name.len() as f64) * 0.1, &item.name))
            .collect::<Vec<_>>()
    );

    let capped = scored_items
        .iter()
        .take(20)
        .map(|(_, item)| *item)
        .collect::<Vec<_>>();

    Json(capped)
}

#[get("/")]
async fn index() -> Option<NamedFile> {
    let p = STATIC_PATH.join(Path::new("index.html"));
    NamedFile::open(p).await.ok()
}

// one possible implementation of walking a directory only visiting files
fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

fn transpose(items: Vec<StoreData>) -> Vec<Item> {
    let mut name2item = HashMap::<&str, Item>::new();
    for store in &items {
        for item in &store.items {
            name2item
                .entry(item.name.as_str())
                .or_insert_with(|| Item {
                    name: item.name.clone(),
                    url: item.url.clone(),
                    price_data: vec![],
                })
                .price_data
                .push(DataForDay {
                    date: store.date,
                    store_id: store.storeId,
                    data: TrimmedParsedItem {
                        price: item.price,
                        compare: item.compare.clone(),
                        sold_in_unit: item.sold_in_unit.clone(),
                        unit_weight: item.unit_weight,
                    },
                });
        }
    }

    name2item.into_values().collect()
}

fn load_items() -> Vec<StoreData> {
    let cache_path = Path::new("cache");
    if cache_path.is_file() {
        return bincode::deserialize_from(BufReader::new(File::open(cache_path).unwrap())).unwrap();
    }

    let mut files: Vec<PathBuf> = vec![];
    visit_dirs(Path::new("../data"), &mut |entry| {
        files.push(entry.path());
    })
    .unwrap();

    let result = files
        .par_iter()
        .progress()
        .map(|f| {
            let date = NaiveDate::parse_from_str(
                f.parent().unwrap().file_name().unwrap().to_str().unwrap(),
                "%Y-%m-%d",
            )
            .unwrap();
            // if date.weekday() != Weekday::Mon {
            //     return None;
            // }
            // let zipped_data = fs::read(f).unwrap();
            let zipped_data = File::open(f).unwrap();
            let decoder = GzDecoder::new(zipped_data);
            let deserializer =
                &mut serde_pickle::de::Deserializer::new(decoder, Default::default());
            match serde_path_to_error::deserialize::<_, Vec<ScrapedItem>>(deserializer) {
                Ok(v) => {
                    // println!("Ok");
                    Some(StoreData {
                        items: v.into_iter().map(parse_item).collect::<Vec<_>>(),
                        storeId: f
                            .parent()
                            .unwrap()
                            .parent()
                            .unwrap()
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .parse::<u32>()
                            .unwrap(),
                        date,
                    })
                }
                Err(e) => {
                    println!("{:?}: {} at {}", f, e, e.path());
                    None
                }
            }
        })
        .flatten()
        .collect::<Vec<StoreData>>();

    bincode::serialize_into(
        io::BufWriter::new(File::create(cache_path).unwrap()),
        &result,
    )
    .unwrap();
    result
}

#[launch]
async fn rocket() -> _ {
    let parsed = load_items();
    let state = ServerState {
        items: transpose(parsed),
    };

    rocket::build()
        .manage(state)
        .mount("/", routes![index, search,])
        .mount("/static", FileServer::from(STATIC_PATH.as_path()))
}
