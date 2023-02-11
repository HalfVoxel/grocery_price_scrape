use crate::types::{
    ComparePrice, ComparePriceVariants, PriceUnit, ProductType, PromotionOrFalse, Promotions,
    ScrapedItem, SoldInUnit,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{map, opt, success},
    sequence::{preceded, tuple},
    IResult,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
#[ts(export)]
pub struct ParsedItem {
    pub name: String,
    pub url: String,
    pub price: f32,
    pub compare: Option<ComparePrice>,
    pub product_type: Option<ProductType>,
    pub sold_in_unit: SoldInUnit,
    pub unit_weight: Option<f32>,
}

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
#[ts(export)]
pub struct StoreData {
    pub storeId: u32,
    pub date: chrono::NaiveDate,
    pub items: Vec<ParsedItem>,
}

fn comma_float(input: &str) -> IResult<&str, f32> {
    let parser = tuple((digit1, opt(preceded(tag(","), digit1))));
    let mut parser = map(parser, |(a, b)| {
        format!("{}.{}", a, b.unwrap_or("0"))
            .parse::<f32>()
            .unwrap()
    });
    parser(input)
}

#[test]
pub fn test_comma_float() {
    assert_eq!(comma_float("1,2"), Ok(("", 1.2)));
    assert_eq!(comma_float("23,212"), Ok(("", 23.212)));
    assert_eq!(comma_float("23"), Ok(("", 23.0)));
}

fn price_unit(input: &str) -> IResult<&str, PriceUnit> {
    preceded(
        tag("kr/"),
        alt((
            preceded(tag("kg, ätklar"), success(PriceUnit::PerKgEdible)),
            preceded(tag("meter"), success(PriceUnit::PerMeter)),
            preceded(tag("kg u. spad"), success(PriceUnit::PerKgWithoutLiquid)),
            preceded(tag("kg"), success(PriceUnit::PerKg)),
            preceded(
                tag("liter, drickklar"),
                success(PriceUnit::PerLiterDrinkable),
            ),
            preceded(tag("liter"), success(PriceUnit::PerLiter)),
            preceded(tag("tvätt"), success(PriceUnit::PerLaundry)),
            preceded(tag("dos"), success(PriceUnit::PerDose)),
            preceded(tag("portion"), success(PriceUnit::PerPortion)),
            preceded(tag("st"), success(PriceUnit::PerItem)),
            preceded(tag("disk"), success(PriceUnit::PerWash)),
            preceded(
                tag("lit exkl. pant"),
                success(PriceUnit::PerLiterExcludingDeposit),
            ),
        )),
    )(input)
}

#[test]
pub fn test_price_unit() {
    assert_eq!(price_unit("kr/meter"), Ok(("", PriceUnit::PerMeter)));
    assert_eq!(price_unit("kr/kg"), Ok(("", PriceUnit::PerKg)));
}

fn compare_price(i: &str) -> IResult<&str, ComparePrice> {
    let (i, _) = tag("Jfr-pris ")(i)?;
    let (i, price) = comma_float(i)?;
    let (i, _) = tag(" ")(i)?;
    let (i, unit) = price_unit(i)?;
    Ok((
        i,
        ComparePrice {
            code: Some(unit),
            price,
            priceText: None,
        },
    ))
}

#[test]
pub fn test_compare_price() {
    assert_eq!(
        compare_price("Jfr-pris 69,90 kr/kg"),
        Ok((
            "",
            ComparePrice {
                code: Some(PriceUnit::PerKg),
                price: 69.90,
                priceText: None
            }
        ))
    );
}

pub fn parse_item(item: ScrapedItem) -> ParsedItem {
    //  () .map(|x| x.parse::<f32>())
    let compare = match item.compare {
        Some(ComparePriceVariants::ComparePrice(x)) => {
            let mut price = x;
            match item.promotions {
                Some(Promotions {
                    priorityPromotion: Some(PromotionOrFalse::Promotion(promotion)),
                    ..
                }) => {
                    if let Some(promotion_price) = &promotion.comparePriceTextWithDeposit {
                        if let Ok(("", prom)) = compare_price(promotion_price) {
                            if prom.price < price.price {
                                price = prom;
                            }
                        }
                    }
                }

                Some(Promotions {
                    remainingPromotions,
                    ..
                }) => {
                    for promotion in remainingPromotions {
                        if let Some(promotion_price) = &promotion.comparePriceTextWithDeposit {
                            if let Ok(("", prom)) = compare_price(promotion_price) {
                                if prom.price < price.price {
                                    price = prom;
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
            Some(price)
        }
        _ => None,
    };

    ParsedItem {
        name: item.name,
        price: item.price,
        product_type: item.productType,
        unit_weight: item.unitWeight,
        sold_in_unit: item.soldInUnit,
        url: format!("https://handla.ica.se/handla/produkt/{}", &item.slug),
        compare,
    }
}
