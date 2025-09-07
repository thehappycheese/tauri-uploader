mod auth;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
#[tauri::command]
async fn login()->Result<auth::UserInfo, ()>{
    let a = auth::AzureAuth::new(
        "",
        "",
    ).map_err(|_|())?;
    let access_token = a.authenticate().await.map_err(|_|())?;
    println!("Got access token!");
    let user_info = a.get_user_info(&access_token).await.map_err(|_|())?;
    println!("got user info {user_info:?}");
    Ok(user_info)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            login
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
