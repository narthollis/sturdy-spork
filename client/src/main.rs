use std::env::args;
use spork_client::client;

fn main() {
    let a = args().nth(1).unwrap().parse::<usize>().unwrap();
    let b = args().nth(2).unwrap().parse::<usize>().unwrap();

    println!("Hello, world! I am Client! {:?}", client(a, b));
}
