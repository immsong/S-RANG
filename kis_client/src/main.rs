use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::env;

#[derive(Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 최상단 폴더에 있는 .env 파일을 읽어옵니다.
    dotenv::from_filename("../.env").ok();

    // 환경 변수에서 키 추출
    let app_key =
        env::var("KIS_MOCK_APP_KEY").expect("[Error] .env 파일에 KIS_MOCK_APP_KEY가 없습니다.");
    let app_secret = env::var("KIS_MOCK_APP_SECRET")
        .expect("[Error] .env 파일에 KIS_MOCK_APP_SECRET이 없습니다.");

    // 한국투자증권 모의투자 토큰 발급 URL
    let url = "https://openapivts.koreainvestment.com:29443/oauth2/tokenP";

    // API에 보낼 JSON 바디
    let body = json!({
        "grant_type": "client_credentials",
        "appkey": app_key,
        "appsecret": app_secret
    });

    let client = Client::new();
    println!("한투(모의) 서버에 접속 토큰을 요청합니다");

    let res = client.post(url).json(&body).send().await?;

    // 결과 확인
    if res.status().is_success() {
        let token_res: TokenResponse = res.json().await?;
        println!(
            "접속 토큰이 정상적으로 발급되었습니다. (토큰: {}, 유형 : {})",
            token_res.access_token, token_res.token_type
        );
        println!("만료 시간(초): {}", token_res.expires_in);
    } else {
        let error_text = res.text().await?;
        println!("토큰 발급 에러: {}", error_text);
    }

    Ok(())
}
