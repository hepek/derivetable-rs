use derivetable::*;

#[derivetable(Clone, Debug)]
#[derive(Table, Debug, Clone)]
pub struct Person {
    #[index]
    name: String,
    #[index]
    surname: String,
    age: usize,
    height: f32,
    #[unique]
    ident: u64,
}

fn main() {
    let mut test = PersonTable::new();

    println!("{:?}", 
        test.insert(Person { name: "Ime".to_string(),  surname: "Prezime".to_string(), age: 10, height: 1.32, ident: 123 }));
    println!("{:?}",
        test.insert(Person { name: "Ime1".to_string(), surname: "Prezime".to_string(), age: 32, height: 1.87, ident: 124 }));
    println!("{:?}",
        test.insert(Person { name: "Ime2".to_string(), surname: "Prezime".to_string(), age: 46, height: 1.32, ident: 125 }));
    println!("{:?}", //Err(id) -- ident constraint not met
        test.insert(Person { name: "Ime3".to_string(), surname: "Prezime".to_string(), age: 12, height: 1.32, ident: 123 }));

    test.remove(&0);

    println!("{:?}", // Ok
        test.insert(Person { name: "Ime3".to_string(), surname: "Prezime".to_string(), age: 12, height: 1.32, ident: 123 }));

    for p in test.range_by_name(.."Ime2".to_string()).rev() {
        println!("P: {:?}", p);
    }

}
