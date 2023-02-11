#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(TS, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub enum PriceUnit {
    #[serde(rename = "pke")]
    PerKgEdible, // kr/kg, ätklar
    #[serde(rename = "pm")]
    PerMeter, // kr/meter
    #[serde(rename = "pkg")]
    PerKg, // kr/kg
    #[serde(rename = "pli")]
    PerLiter, // kr/liter
    #[serde(rename = "pld")]
    PerLiterDrinkable, // kr/liter, drickklar
    #[serde(rename = "pla")]
    PerLaundry, // kr/tvätt
    #[serde(rename = "pdo")]
    PerDose, // kr/dos
    #[serde(rename = "por")]
    PerPortion, // kr/portion
    #[serde(rename = "ppi")]
    PerItem, // kr/st
    #[serde(rename = "pwa")]
    PerWash, // kr/disk
    #[serde(rename = "kwl")]
    PerKgWithoutLiquid, // kr/kg u. spad
    #[serde(rename = "plx")]
    PerLiterExcludingDeposit, // kr/lit exkl. pant
    #[serde(rename = "grm")]
    Gram, // Grams, though weirdly it seems to be only "grams" not "kr/g"??
    #[serde(rename = "kgm")]
    Kg, // Kg, though weirdly it seems to be only "kg" not "kr/kg"??
    Invalid,
    #[allow(non_camel_case_types)]
    XX_PER_L_READY_DEPOSIT,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComparePriceVariants {
    ComparePrice(ComparePrice),
    Number(u32),
}

#[derive(TS, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[ts(export)]
pub struct ComparePrice {
    pub code: Option<PriceUnit>,
    pub price: f32,
    pub priceText: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InCategoryPath {
    pub id: String,
    pub level: u32,
    pub name: String,
    pub slug: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InCategory {
    pub id: String,
    pub level: u32,
    pub name: String,
    pub path: Vec<InCategoryPath>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Marking {
    pub code: String,
    pub title: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Markings {
    pub environmental: Option<Vec<Marking>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CategoryRef {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum RelatedRootCategory {
    Empty {},
    CategoryRef(CategoryRef),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Related {
    pub inCategories: Vec<InCategory>,
    pub markings: Markings,
    pub promotions: Option<String>,
    pub related: Vec<Related>,
    pub rootCategory: RelatedRootCategory,
    pub sku: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum PriceText {
    String(String),
    Number(u32),
}

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
#[ts(export)]
pub enum SoldInUnit {
    #[serde(rename = "kgm")]
    Kgm,
    #[serde(rename = "pce")]
    Pce,
}

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
#[ts(export)]
pub enum ProductType {
    #[serde(rename = "food")]
    Food,
    #[serde(rename = "non-food")]
    NonFood,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Promotion {
    pub comparePrice: Option<f32>,
    pub comparePriceTextWithDeposit: Option<String>,
    pub comparePriceTextWithoutDeposit: Option<String>,
    pub count: i32,
    pub forLoggedIn: bool,
    pub hasLongValidity: bool,
    pub hasShortValidity: bool,
    pub id: i32,
    pub isWeightedFixedPrice: Option<bool>,
    pub maxItemText: Option<String>,
    pub name: String,
    pub numberOfItemsInPromotion: i32,
    pub offerId: Option<String>,
    pub price: f32,
    // splashProperties
    pub supressComparePriceRange: Option<bool>,
    #[serde(alias = "type")]
    pub promotionType: String,
    pub validTo: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum PromotionOrFalse {
    Promotion(Promotion),
    Bool(bool),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Promotions {
    pub priorityPromotion: Option<PromotionOrFalse>,
    #[serde(default)]
    pub remainingPromotions: Vec<Promotion>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScrapedItem {
    pub availableUnits: Vec<String>,
    pub cloudinaryImageId: String,
    pub compare: Option<ComparePriceVariants>,
    pub deposit: f32,
    pub descriptionLong: Option<String>,
    pub gtin: u64,
    pub inCategories: Vec<InCategory>,
    pub ingredientsText: Option<String>,
    pub markings: Markings,
    pub name: String,
    pub nutritionalText: Option<String>,
    pub pharmaAdviceFlag: bool,
    pub price: f32,
    pub priceMinusDeposit: f32,
    pub priceText: PriceText,
    pub productDisclaimer: Option<String>,
    pub productType: Option<ProductType>,
    pub promotions: Option<Promotions>,
    pub related: Vec<Related>,
    pub rootCategory: RelatedRootCategory,
    pub sku: String,
    pub slug: String,
    pub soldInUnit: SoldInUnit,
    pub unitWeight: Option<f32>,
    pub vat: u32,
}
