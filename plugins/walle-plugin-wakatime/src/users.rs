use std::collections::HashMap;
use tokio::{
    fs::{remove_file, File},
    io::{AsyncReadExt, AsyncWriteExt},
};

const USERS_FILE: &str = "wakatime.json";

type Users = HashMap<String, HashMap<String, String>>;

pub async fn load_users() -> Result<Users, String> {
    match File::open(USERS_FILE).await {
        Ok(mut file) => {
            let mut file_data = Vec::new();
            file.read_to_end(&mut file_data)
                .await
                .map_err(|e| e.to_string())?;
            serde_json::from_str(&String::from_utf8(file_data).unwrap()).map_err(|e| e.to_string())
        }
        Err(_) => {
            File::create(USERS_FILE).await.ok();
            Ok(Users::default())
        }
    }
}

pub async fn save_users(users: &Users) -> Result<(), String> {
    remove_file(USERS_FILE).await.ok();
    let mut file = File::create(USERS_FILE).await.map_err(|e| e.to_string())?;
    file.write_all(serde_json::to_string(&users).unwrap().as_bytes())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
