use argon2::password_hash::{PasswordHasher, SaltString};
use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;

fn main() {
    let password = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: hash_password <password>");
        std::process::exit(1);
    });

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string();

    println!("{hash}");
}
