use std::collections::HashMap;
use std::io::BufRead;

pub trait VerifyPassword {
    fn check_password(&self, username: &str, password: &str) -> Option<u32>;
}

pub struct ArgonPasswordsInFile {
    users: HashMap<String, (u32, String)>,
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
            let mut linesplit = line.split('~');
            let Some(uid) = linesplit.next() else {
                continue;
            };
            let Some(username) = linesplit.next() else {
                continue;
            };
            let Some(password) = linesplit.next() else {
                continue;
            };
            if linesplit.next().is_some() {
                continue;
            }
            let Ok(uid) = uid.parse() else {
                continue;
            };
            users.insert(username.to_string(), (uid, password.to_string()));
            debug_loaded += 1;
        }
        println!("LOADED {debug_loaded} CREDENTIALS");
        Ok(ArgonPasswordsInFile{
            users,
        })
    }
}

impl VerifyPassword for ArgonPasswordsInFile {
    fn check_password(&self, username: &str, password: &str) -> Option<u32> {
        let Some((uid, pass_verify)) = self.users.get(username) else {
            return None;
        };
        let Ok(verified) = argon2::verify_encoded(&pass_verify, password.as_bytes()) else {
            return None;
        };
        if verified {
            Some(*uid)
        } else {
            None
        }
    }
}
