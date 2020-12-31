use derivetable::*;
///Scratch
//////

#[derivetable(Clone, Debug)]
#[derive(Table, Debug, Clone)]
pub struct Person {
    #[index]
    name: String,
    #[hindex]
    surname: String,
    age: usize,
    height: f32,
    #[unique]
    ident: u64,
}

#[test]
fn insert_and_remove() {
    let mut test = PersonTable::new();
    let test_row = Person { name: "Name".to_string(), surname: "Surname".to_string(), age: 35, height: 1.78, ident: 1234 };

    assert!(test.insert(Person { ident: 1234, ..test_row.clone() }).is_ok());
    assert!(test.insert(Person { ident: 456,  name: "Milan".to_string(), ..test_row.clone() }).is_ok());
    assert!(test.insert(Person { ident: 55,   name: "Goran".to_string(), ..test_row.clone() }).is_ok());
    let res = test.insert(Person { ident: 1234, name: "Ivan".to_string(),  ..test_row.clone() });
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), 0);
    assert!(test.remove(0).is_some());
    assert!(test.insert(Person { ident: 1234, name: "Zoran".to_string(),  ..test_row.clone() }).is_ok());
}


// main test compares sqlite performance vs derivetable performance on a simple task
// it reads nyc yellow cab trip records file from stdin, parses the CSV and inserts into a table
// then performs some simple queries


use csv;
use serde_derive;

struct DateTimeVisitor;

use serde::de;
use std::fmt;

impl<'de> de::Visitor<'de> for DateTimeVisitor {
    type Value = chrono::NaiveDateTime;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string representing YYYY-mm-dd HH:MM:SS time")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: de::Error 
    {
        match chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
            Ok(t) => Ok(t),
            Err(_) => Err(de::Error::invalid_value(de::Unexpected::Str(s), &self)),
        }
    }
}

fn date_time_parse<'de, D>(d: D) -> Result<chrono::NaiveDateTime, D::Error> 
    where
        D: de::Deserializer<'de>
{
    d.deserialize_str(DateTimeVisitor)
}

fn money_parse<'de, D>(d: D) -> Result<u32, D::Error> 
    where
        D: de::Deserializer<'de>
{
    use serde::Deserialize;
    let f: f32 = Deserialize::deserialize(d)?;
    Ok((f * 100.0) as u32)
}

#[derivetable(Debug)]
#[derive(Table, Debug, Clone, serde_derive::Deserialize)]
#[allow(non_snake_case)]
struct CabTrip {
    VendorID: Option<u32>,
    #[serde(deserialize_with = "date_time_parse")]
    #[index]
    tpep_pickup_datetime: chrono::NaiveDateTime,
    #[serde(deserialize_with = "date_time_parse")]
    #[index]
    tpep_dropoff_datetime: chrono::NaiveDateTime,
    passenger_count: Option<u8>,
    trip_distance: f32,
    RatecodeID: Option<u32>,
    store_and_fwd_flag: String,
    PULocationID: Option<u32>,
    DOLocationID: Option<u32>,
    payment_type: Option<u32>,
    #[serde(deserialize_with = "money_parse")]
    #[index]
    fare_amount: u32,
    extra: Option<f32>,
    mta_tax: Option<f32>,
    tip_amount: Option<f32>,
    tolls_amount: Option<f32>,
    improvement_surcharge: Option<f32>,
    total_amount: Option<f32>,
    congestion_surcharge: Option<f32>,
}

fn main() {
    let mut rdr = csv::Reader::from_reader(std::io::stdin());
    println!("starting, {}", rdr.has_headers());
    println!("sizeof Row: {}", std::mem::size_of::<CabTrip>());
    //let mut trips = vec![];

    let now = std::time::Instant::now();
    let mut tript = CabTripTable::new();
    for record in rdr.deserialize()  {
        let r: CabTrip = record.unwrap();
        //trips.push(r);
        tript.insert(r).unwrap();
    }
    
    let num = tript.data.len(); //trips.len();
    println!("total trips: {}, time: {}s", num, now.elapsed().as_secs_f64());
    
    /*
    let now = std::time::Instant::now();
    let mut small_fares = 0usize;
    for record in trips.iter() {
        if record.fare_amount < 500 {
            small_fares += 1;
        }
    }
    println!("Num fares < 10$: {}, vec iter time: {}ms", small_fares, now.elapsed().as_secs_f64()*1000.0);

    let now = std::time::Instant::now();
    for record in trips {
        tript.insert(record).unwrap();
    }
    */

    println!("table insert time: {}s, per record: {}ms", now.elapsed().as_secs_f64(), now.elapsed().as_secs_f64()*1000.0/(num as f64));

    let mut fare = 0usize;
    let now = std::time::Instant::now();
    for record in tript.iter() {
        fare += record.fare_amount as usize;
    }

    println!("total fare collected: {}USD, iter time: {}ms", fare as f64/100.0, now.elapsed().as_secs_f64()*1000.0);


    /*
    let now = std::time::Instant::now();
    let mut small_fares = 0usize;
    for (_, _) in tript.range_by_fare_amount(0..500) {
        small_fares += 1;
    }
    println!("Num fares < 5$: {}, iter time: {}ms", small_fares, now.elapsed().as_secs_f64()*1000.0);

    let now = std::time::Instant::now();
    let mut exact_amount = 0usize;
    for (_, _) in tript.get_by_fare_amount(&1234) {
        exact_amount += 1;
    }

    println!("Num fares == 12.34$: {}, iter time: {}ms", exact_amount, now.elapsed().as_secs_f64()*1000.0);
    */
}
