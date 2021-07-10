use derivetable::*;
use rusqlite::*;

#[derive(Table, Debug, Clone)]
#[derivetable(Clone, Debug)]
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

#[derive(Table, Debug, Clone, serde_derive::Deserialize)]
#[derivetable(Debug)]
#[allow(non_snake_case)]
struct CabTrip {
    #[hindex]
    VendorID: Option<u32>,
    #[serde(deserialize_with = "date_time_parse")]
    #[index]
    tpep_pickup_datetime: chrono::NaiveDateTime,
    #[serde(deserialize_with = "date_time_parse")]
    #[index]
    tpep_dropoff_datetime: chrono::NaiveDateTime,
    passenger_count: Option<u8>,
    trip_distance: f64,
    RatecodeID: Option<u32>,
    store_and_fwd_flag: String,
    PULocationID: Option<u32>,
    DOLocationID: Option<u32>,
    payment_type: Option<u32>,
    #[serde(deserialize_with = "money_parse")]
    #[index]
    fare_amount: u32,
    extra: Option<f64>,
    mta_tax: Option<f64>,
    tip_amount: Option<f64>,
    tolls_amount: Option<f64>,
    improvement_surcharge: Option<f64>,
    total_amount: Option<f64>,
    congestion_surcharge: Option<f64>,
}

fn main() {
    let mut rdr = csv::Reader::from_reader(std::io::stdin());
    println!("starting, {}", rdr.has_headers());
    println!("sizeof Row: {}", std::mem::size_of::<CabTrip>());
    let mut trips = vec![];

    let now = std::time::Instant::now();
    for record in rdr.deserialize()  {
        let r: CabTrip = record.unwrap();
        trips.push(r);
    }
    let trips2 = trips.clone();
    println!("================= DERIVETABLE =====================");
    
    let num = trips.len();
    println!("total trips: {}, time: {}s", num, now.elapsed().as_secs_f64());
    
    let mut tript = CabTripTable::new();
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

    println!("table insert time: {}s, per record: {}ms", now.elapsed().as_secs_f64(), now.elapsed().as_secs_f64()*1000.0/(num as f64));

    let mut fare = 0usize;
    let now = std::time::Instant::now();
    for record in tript.iter() {
        fare += record.fare_amount as usize;
    }

    println!("total fare collected: {}USD, iter time: {}ms", fare as f64/100.0, now.elapsed().as_secs_f64()*1000.0);

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


    println!("================= SQLITE3 MEMORY =====================");
    // Sqlite3 memory comparison
    let conn = Connection::open_in_memory().unwrap();

    conn.execute(
        "CREATE TABLE CabTrip (
            id  INTEGER PRIMARY KEY,
            VendorID    INTEGER,
            tpep_pickup_datetime TEXT,
            tpep_dropoff_datetime TEXT,
            passenger_count INTEGER,
            trip_distance REAL,
            RatecodeID INTEGER,
            store_and_fwd_flag TEXT,
            PULocationID INTEGER,
            DOLocationID INTEGER,
            payment_type INTEGER,
            fare_amount INTEGER,
            extra INTEGER,
            mta_tax INTEGER,
            tip_amount REAL,
            tolls_amount REAL,
            improvement_surcharge REAL,
            total_amount REAL,
            congestion_surcharge REAL
            );",
        params![],
    ).unwrap();

    conn.execute("CREATE INDEX vendorid on CabTrip(VendorId);", params![]).unwrap();
    conn.execute("CREATE INDEX pickup on CabTrip( tpep_pickup_datetime );", params![]).unwrap();
    conn.execute("CREATE INDEX dropoff on CabTrip( tpep_dropoff_datetime );", params![]).unwrap();
    conn.execute("CREATE INDEX amt on CabTrip( fare_amount );", params![]).unwrap();

    let now = std::time::Instant::now();
    for record in trips2 {
        let pickup = record.tpep_pickup_datetime.format("%Y-%m-%d %H:%M:%S").to_string();
        let dropoff = record.tpep_dropoff_datetime.format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute("INSERT INTO CabTrip ( VendorID, tpep_pickup_datetime, tpep_dropoff_datetime, passenger_count, trip_distance, RatecodeID,
            store_and_fwd_flag, PULocationID, DOLocationID, payment_type, fare_amount,
            extra, mta_tax, tip_amount, tolls_amount, improvement_surcharge,
            total_amount, congestion_surcharge) VALUES 
            ( ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18 );",
            params![
            &record.VendorID,
            &pickup,
            &dropoff,
            &record.passenger_count,
            &record.trip_distance,
            &record.RatecodeID,
            &record.store_and_fwd_flag,
            &record.PULocationID,
            &record.DOLocationID,
            &record.payment_type,
            &record.fare_amount,
            &record.extra,
            &record.mta_tax,
            &record.tip_amount,
            &record.tolls_amount,
            &record.improvement_surcharge,
            &record.total_amount,
            &record.congestion_surcharge,
            ]).unwrap();
    }

    println!("SQLITE3 insert time: {}s, per record: {}ms", now.elapsed().as_secs_f64(), now.elapsed().as_secs_f64()*1000.0/(num as f64));

    let now = std::time::Instant::now();
    let mut stmt = conn.prepare("SELECT * from CabTrip WHERE fare_amount < 500;").unwrap();

    let mut small_fares = 0usize;
    let mut q = stmt.query(params![]).unwrap();
    while let Some(_row) = q.next().unwrap() {
        small_fares += 1;
    }
    println!("Num fares < 5$: {}, iter time: {}ms", small_fares, now.elapsed().as_secs_f64()*1000.0);

    let now = std::time::Instant::now();
    let mut exact_amount = 0usize;
    let mut stmt = conn.prepare("SELECT * from CabTrip WHERE fare_amount = 1234;").unwrap();
    let mut q = stmt.query(params![]).unwrap();
    while let Some(_row) = q.next().unwrap() {
        exact_amount += 1;
    }

    println!("Num fares == 12.34$: {}, iter time: {}ms", exact_amount, now.elapsed().as_secs_f64()*1000.0);
}
