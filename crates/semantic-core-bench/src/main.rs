fn main() {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "all".to_string());
    match semantic_core_bench::run_benchmark(&command) {
        Ok(text) => {
            println!("{text}");
        }
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(2);
        }
    }
}
