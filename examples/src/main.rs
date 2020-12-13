use derivetable::*;

#[derive(Table, Debug)]
struct Person {
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
    let test = PersonTable::new();
    println!("persontable.idx_name: {:?}", test.idx_name);
}
