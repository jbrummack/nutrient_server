use flate2::read::GzDecoder;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, BufRead};
use std::time::Duration;
static STORE: OnceLock<HashMap<String, String>> = OnceLock::new();
use actix_web::{get, web, App, HttpServer};
use local_ip_address::local_ip;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::sync::OnceLock;

#[derive(Debug, Serialize, Deserialize)]
struct FoodResponseV0 {
    id: String,
    nutrients: Vec<String>,
    urls: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct FoodResponseV1 {
    productname: String,
    id: String,
    ingredients: String,
    brands: String,
    categories: String,
    quantity: String,
    nutrients: Vec<String>,
    urls: Vec<String>,
}

#[get("/api/nutrient/{key}")]
async fn get_value(path: web::Path<String>) -> String {
    match fetch_value(path.to_string()) {
        Some(s) => s.to_owned(),
        None => String::from("Not found"),
    }
}

#[get("/api/nutrients/{key}")]
async fn get_values(path: web::Path<String>) -> String {
    match fetch_value(path.to_string()) {
        Some(s) => s.to_owned(),
        None => String::from("Not found"),
    }
}

fn fetch_value(key: String) -> Option<&'static String> {
    match STORE.get() {
        Some(store) => match store.get(&key) {
            Some(value) => Some(value),
            None => None,
        },
        None => None,
    }
}

fn parse_header(l: String) -> Vec<String> {
    l.split("\t")
        .into_iter()
        .map(|substr| substr.to_string())
        .collect()
}

fn parse_data(l: String, header: Vec<String>) -> FoodResponseV1 {
    let mut nutrients: Vec<String> = Vec::new();
    let mut urls: Vec<String> = Vec::new();
    let mut code: String = String::new();
    let mut product_name: String = String::new();
    let mut ingredients: String = String::new();
    let mut brands: String = String::new();
    let mut categories: String = String::new();
    let mut quantity: String = String::new();

    l.split("\t").into_iter().enumerate().for_each(|entry| {
        let word = entry.1;
        let nr = entry.0;
        let labels = &header;
        if nr <= header.len() {
            let label = &labels[nr];
            let combined = format!("{}:{}", label, word);
            if &label.contains("100g") == &true && word.len() > 0 {
                nutrients.push(combined.clone());
            }
            if &label.contains("url") == &true && word.len() > 0 {
                urls.push(combined);
            }
            if &label.contains("product_name") == &true && word.len() > 0 {
                product_name = word.to_owned();
            }
            if &label.contains("ingredients") == &true && word.len() > 0 {
                ingredients = word.to_owned();
            }
            if &label.contains("brands") == &true && word.len() > 0 {
                brands = word.to_owned();
            }
            if &label.contains("categories") == &true && word.len() > 0 {
                categories = word.to_owned();
            }
            if &label.contains("quantity") == &true && word.len() > 0 {
                quantity = word.to_owned();
            }
            if nr == 0 {
                code = word.to_owned()
            }
        }
    });
    FoodResponseV1 {
        productname: product_name,
        id: code.clone(),
        nutrients: nutrients,
        urls: urls,
        ingredients: ingredients,
        brands: brands,
        categories: categories,
        quantity: quantity,
    }
}

async fn setup() -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    // Step 1: Download the .gz file as a byte stream

    let url = "https://static.openfoodfacts.org/data/en.openfoodfacts.org.products.csv.gz"; // Replace with the actual URL
    let client = Client::builder()
        .timeout(Duration::from_secs(60 * 30))
        .build()
        .unwrap();
    let response = client
        .get(url)
        .send()
        .await
        .expect("failed to download file");

    // Ensure the response is successful
    /*  if !response.is_ok() {
    return Err(Box::new(io::Error::new(
        io::ErrorKind::Other,
        "Failed to download file",
    )));
    }*/

    // Step 2: Decompress the byte stream
    let binding = response.bytes().await.expect("failed to read bytes!");
    let mut decoder = GzDecoder::new(binding.as_ref());
    // Step 3: Split the decompressed data into lines
    let reader = io::BufReader::new(&mut decoder);
    let mut header: Vec<String> = Vec::new();
    let mut database: HashMap<String, String> = HashMap::new();
    for line in reader.lines().enumerate() {
        let mut return_struct = FoodResponseV1::default();
        match line {
            (0, Ok(l)) => {
                println!("HEADER: {}", l);
                header = l
                    .split("\t")
                    .into_iter()
                    .map(|substr| substr.to_string())
                    .collect();
            } // Process each line (e.g., print it)
            (_lnr, Ok(l)) => {
                return_struct = parse_data(l, header.clone());
            }
            (lnr, Err(e)) => eprintln!("Error reading line {lnr}: {}", e),
        }

        database.insert(
            return_struct.id.clone(),
            serde_json::to_string(&return_struct)?,
        );
    }

    Ok(database)
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "0");
    std::env::set_var("RUST_LOG", "debug");
    let my_ip = local_ip().unwrap();
    dbg!(my_ip);
    env_logger::init();
    let storage = async { setup().await.expect("Failed to initialize!") }.await;
    STORE.get_or_init(|| storage);
    HttpServer::new(|| App::new().service(get_value))
        .bind((my_ip.to_string(), 3333))?
        .run()
        .await
}

//STREAM SCHEMA:
//reqwest -> gzip -> decode lines -> hashmap
