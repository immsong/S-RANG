use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::error::Error;

#[derive(Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    expires_in: u32,
}

#[derive(Deserialize, Debug)]
struct ApprovalResponse {
    approval_key: String,
}

/// REST API용 접속 토큰 발급
pub async fn get_access_token(
    client: &Client,
    app_key: &str,
    app_secret: &str,
) -> Result<String, Box<dyn Error>> {
    let url = "https://openapivts.koreainvestment.com:29443/oauth2/tokenP";

    let body = json!({
        "grant_type": "client_credentials",
        "appkey": app_key,
        "appsecret": app_secret
    });

    let res = client.post(url).json(&body).send().await?;

    if res.status().is_success() {
        let token_res: TokenResponse = res.json().await?;
        println!(
            "Success to get REST API access token (expired: {} seconds)",
            token_res.expires_in
        );
        Ok(token_res.access_token)
    } else {
        let err_msg = res.text().await?;
        Err(format!("Failed to get REST API access token: {}", err_msg).into())
    }
}

/// 웹소켓 접속용 승인키(Approval Key) 발급
pub async fn get_approval_key(
    client: &Client,
    app_key: &str,
    app_secret: &str,
) -> Result<String, Box<dyn Error>> {
    let url = "https://openapivts.koreainvestment.com:29443/oauth2/Approval";

    // 주의: 웹소켓 승인키 요청 시에는 'appsecret' 대신 'secretkey'를 키 이름으로 사용합니다. (KIS API 규격)
    let body = json!({
        "grant_type": "client_credentials",
        "appkey": app_key,
        "secretkey": app_secret
    });

    let res = client.post(url).json(&body).send().await?;

    if res.status().is_success() {
        let approval_res: ApprovalResponse = res.json().await?;
        println!("Success to get WebSocket approval key (Approval Key)");
        Ok(approval_res.approval_key)
    } else {
        let err_msg = res.text().await?;
        Err(format!("Failed to get WebSocket approval key: {}", err_msg).into())
    }
}
