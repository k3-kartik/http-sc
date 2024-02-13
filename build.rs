macro_rules! build_print {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() {
    let env_file = std::fs::read_to_string("./.env").unwrap();
    build_print!("env file: {}", env_file);
}
