fn main() {
    let array = lib::power_levels(300, 9995);
    println!("{:?}", lib::find_max(&lib::sum_window(&array, 3)).coords);
    println!("{:?}", (1..=300)
             .map(|i| lib::find_max(&lib::sum_window(&array, i)))
             .enumerate()
             .max_by_key(|(_, m)| m.value)
             .map(|(i, m)| (m.coords.0, m.coords.1, i + 1))
             .unwrap());
}
