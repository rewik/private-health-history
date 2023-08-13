use rand::Fill;

fn main() -> () {
    let p1 = rpassword::prompt_password("Enter password: ").unwrap();
    let p2 = rpassword::prompt_password("Enter password again: ").unwrap();
    if p1 != p2 {
        println!("ERROR: password mismatch!");
        return ();
    }
    let mut rng = rand::thread_rng();
    let mut salt = [0u8; 16];
    //rng.fill_bytes(&salt);
    salt.try_fill(&mut rng).unwrap();
    let hash = argon2::hash_encoded(p1.as_bytes(), &salt, &argon2::Config::default()).unwrap();
    println!("{}", hash);
}
