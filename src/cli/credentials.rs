use std::{env, fs, io::ErrorKind};

const KEY_NAME: &str = "PROVE2ME_API_KEY";

pub fn read_key() -> Result<String, &'static str> {
    let key = match env::var(KEY_NAME) {
        Ok(value) => value,
        Err(env::VarError::NotUnicode(_)) => return Err("PROVE2ME_API_KEY is not valid text"),
        Err(env::VarError::NotPresent) => read_dotenv_key()?,
    };
    if key.trim().is_empty() {
        return Err("set PROVE2ME_API_KEY in the environment or a local .env file");
    }
    Ok(key)
}

fn read_dotenv_key() -> Result<String, &'static str> {
    let text = match fs::read_to_string(".env") {
        Ok(text) => text,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(String::new()),
        Err(_) => return Err("could not read .env"),
    };
    for (name, value) in text.lines().filter_map(|line| line.split_once('=')) {
        if name.trim() == KEY_NAME {
            let value = value.trim();
            if value.contains(char::is_whitespace) || value.starts_with(['\'', '"']) {
                return Err("use an unquoted PROVE2ME_API_KEY value in .env");
            }
            return Ok(value.to_owned());
        }
    }
    Ok(String::new())
}
