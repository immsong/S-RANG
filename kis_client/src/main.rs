mod auth; // auth.rs 모듈을 불러옵니다.

use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use serde_json::json;
use std::env;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::from_filename("../.env").ok();

    let app_key = env::var("KIS_MOCK_APP_KEY").expect("KIS_MOCK_APP_KEY not found");
    let app_secret = env::var("KIS_MOCK_APP_SECRET").expect("KIS_MOCK_APP_SECRET not found");

    let client = Client::new();

    println!("KIS Client is starting...");

    // REST API용 토큰 발급 (나중에 계좌 조회 등 다른 목적을 위해 대비)
    let _access_token = auth::get_access_token(&client, &app_key, &app_secret).await?;

    // 웹소켓 전용 승인키 발급
    let approval_key = auth::get_approval_key(&client, &app_key, &app_secret).await?;

    println!("All authentication keys are ready.");
    // 보안상 터미널에 키 값을 직접 출력하지는 않습니다.

    // KIS 모의투자 웹소켓 서버 URL
    let ws_url = "ws://ops.koreainvestment.com:31000";

    println!("Connecting to {}", ws_url);
    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("Failed to connect to WebSocket server");
    println!("Connected to WebSocket server.");

    // 스트림을 송신(write)과 수신(read)으로 분리
    let (mut write, mut read) = ws_stream.split();

    // 삼성전자(005930) 호가창 데이터 구독(Subscribe) 요청 메시지 포맷팅
    // TR_ID "H0STASP0"는 주식 호가창을 의미합니다.
    let subscribe_msg = json!({
        "header": {
            "approval_key": approval_key,
            "custtype": "P",
            "tr_type": "1", // 1: 구독 등록, 2: 구독 해제
            "content-type": "utf-8"
        },
        "body": {
            "input": {
                "tr_id": "H0STASP0",
                "tr_key": "005930"
            }
        }
    });

    // 구독 요청 발송
    write
        .send(Message::Text(subscribe_msg.to_string().into()))
        .await?;
    println!("Sent subscription request for Samsung Electronics (005930) order book data.");
    println!("Waiting for real-time data... (Press Ctrl+C to exit)");

    // 실시간 데이터 수신 무한 루프
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // 수신된 날것(Raw)의 데이터를 콘솔에 출력
                println!("Received data: {}", text);
            }
            Ok(Message::Ping(ping)) => {
                // 서버에서 연결 유지를 위해 Ping을 보내면 Pong으로 응답
                write.send(Message::Pong(ping)).await?;
            }
            Err(e) => {
                println!("WebSocket receive error: {}", e);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
