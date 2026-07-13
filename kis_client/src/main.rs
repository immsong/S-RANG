mod auth; // auth.rs 모듈을 불러옵니다.

use reqwest::Client;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::from_filename("../.env").ok();

    let app_key = env::var("KIS_MOCK_APP_KEY").expect("KIS_MOCK_APP_KEY not found");
    let app_secret = env::var("KIS_MOCK_APP_SECRET").expect("KIS_MOCK_APP_SECRET not found");

    let client = Client::new();

    println!("KIS Client is starting...");

    // REST API용 토큰 발급 (나중에 계좌 조회 등 다른 목적을 위해 대비)
    let access_token = auth::get_access_token(&client, &app_key, &app_secret).await?;

    // 웹소켓 전용 승인키 발급
    let approval_key = auth::get_approval_key(&client, &app_key, &app_secret).await?;

    println!("All authentication keys are ready.");
    // 보안상 터미널에 키 값을 직접 출력하지는 않습니다.

    Ok(())
}
