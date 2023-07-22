fn main() {
    let gb = qgb::GameBoy::new(&vec![], &vec![]);
    println!("{:#?}", gb.cpu());
}
