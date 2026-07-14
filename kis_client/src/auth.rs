use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

const TOKEN_FILE: &str = "kis_tokens.json";

// 캐시에 저장하고 메모리에서 사용할 통합 인증 구조체
#[derive(Serialize, Deserialize, Debug)]
pub struct AuthTokens {
    pub access_token: String,
    pub approval_key: String,
    pub expires_at: u64,
}

// API 응답 파싱용 내부 구조체
#[derive(Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Deserialize, Debug)]
struct ApprovalResponse {
    approval_key: String,
}

/// 캐시를 확인하고, 없거나 만료되었을 때만 서버에서 키를 새로 발급받습니다.
pub async fn get_auth_tokens(
    client: &Client,
    app_key: &str,
    app_secret: &str,
) -> Result<AuthTokens, Box<dyn Error>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    // 로컬 캐시 파일 읽기 시도
    if let Ok(data) = fs::read_to_string(TOKEN_FILE) {
        if let Ok(cached) = serde_json::from_str::<AuthTokens>(&data) {
            // 여유 시간(1시간 = 3600초)을 두고 만료 여부 확인
            if cached.expires_at > now + 3600 {
                println!("Loaded cached authentication tokens from {}", TOKEN_FILE);
                return Ok(cached);
            }
        }
    }

    println!("Cache is missing or expired. Requesting new tokens from the server...");

    // REST API 접속 토큰 발급
    let token_url = "https://openapivts.koreainvestment.com:29443/oauth2/tokenP";
    let token_body = json!({
        "grant_type": "client_credentials",
        "appkey": app_key,
        "appsecret": app_secret
    });

    let res = client.post(token_url).json(&token_body).send().await?;
    if !res.status().is_success() {
        return Err(format!("Failed to obtain access token: {}", res.text().await?).into());
    }
    let token_res: TokenResponse = res.json().await?;

    // 웹소켓 승인키 발급
    let ws_url = "https://openapivts.koreainvestment.com:29443/oauth2/Approval";
    let ws_body = json!({
        "grant_type": "client_credentials",
        "appkey": app_key,
        "secretkey": app_secret
    });

    let res = client.post(ws_url).json(&ws_body).send().await?;
    if !res.status().is_success() {
        return Err(format!("Failed to obtain approval key: {}", res.text().await?).into());
    }
    let app_res: ApprovalResponse = res.json().await?;

    // 새로운 키를 구조체에 담고 로컬 파일로 저장(캐싱)
    let new_tokens = AuthTokens {
        access_token: token_res.access_token,
        approval_key: app_res.approval_key,
        expires_at: now + token_res.expires_in,
    };

    let cache_data = serde_json::to_string_pretty(&new_tokens)?;
    fs::write(TOKEN_FILE, cache_data)?;
    println!(
        "New authentication tokens issued and cached to local file ({})",
        TOKEN_FILE
    );

    Ok(new_tokens)
}
