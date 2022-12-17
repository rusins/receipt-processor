pub fn print_price(price: u32) -> String {
    format!("{}.{}", price / 100, price % 100)
}