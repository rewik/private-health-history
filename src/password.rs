use std::collections::HashMap;
use std::io::BufRead;

pub trait VerifyPassword {
    fn check_password(&self, username: &str, password: &str) -> bool;
}

pub struct ArgonPasswordsInFile {
    users: HashMap<String, String>,
}

impl ArgonPasswordsInFile {
    pub fn try_from<T: AsRef<std::path::Path>>(file: T) -> Result<ArgonPasswordsInFile, &'static str> {
        let Ok(file) = std::fs::File::open(file) else {
            return Err("unable to read the file");
        };
        let mut users = HashMap::new();
        let mut debug_loaded = 0;
        for line in std::io::BufReader::new(file).lines() {
            if line.is_err() {
                break;
            }
            let line = line.unwrap();
            let Some((username, password)) = line.split_once('~') else {
                continue;
            };
            users.insert(username.to_string(), password.to_string());
            debug_loaded += 1;
        }
        println!("LOADED {debug_loaded} CREDENTIALS");
        Ok(ArgonPasswordsInFile{
            users,
        })
    }
}

impl VerifyPassword for ArgonPasswordsInFile {
    fn check_password(&self, username: &str, password: &str) -> bool {
        let Some(pass_verify) = self.users.get(username) else {
            return false;
        };
        let Ok(verified) = argon2::verify_encoded(&pass_verify, password.as_bytes()) else {
            return false;
        };

        verified
    }
}
