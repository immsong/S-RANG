mod auth;

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

                // 불필요 문자열 건너뛰기
                if text.contains("PINGPONG") || text.contains("SUBSCRIBE") {
                    continue;
                }

                let parts: Vec<&str> = text.split('|').collect();

                // 정상적인 호가창 데이터(H0STASP0)인지 검증
                if parts.len() >= 4 && parts[1] == "H0STASP0" {
                    let data_body = parts[3];
                    let fields: Vec<&str> = data_body.split('^').collect();

                    // 10호가 데이터를 모두 포함하려면 최소 45개 이상의 필드가 있어야 함
                    if fields.len() >= 45 {
                        let time = fields[1];

                        // 10호가 데이터를 담을 배열 초기화
                        let mut ask_prices = [0u32; 10];
                        let mut bid_prices = [0u32; 10];
                        let mut ask_vols = [0u32; 10];
                        let mut bid_vols = [0u32; 10];

                        // 반복문을 통해 1~10호가 데이터를 배열에 삽입
                        for i in 0..10 {
                            ask_prices[i] = fields[3 + i].parse().unwrap_or(0);
                            bid_prices[i] = fields[13 + i].parse().unwrap_or(0);
                            ask_vols[i] = fields[23 + i].parse().unwrap_or(0);
                            bid_vols[i] = fields[33 + i].parse().unwrap_or(0);
                        }

                        // 총 잔량 추출
                        let total_ask_vol: u32 = fields[43].parse().unwrap_or(0);
                        let total_bid_vol: u32 = fields[44].parse().unwrap_or(0);

                        println!(
                            "[Tick: {}] Total Ask Qty: {} | Total Bid Qty: {}",
                            time, total_ask_vol, total_bid_vol
                        );
                        println!("  -> Ask Prices 1-10: {:?}", ask_prices);
                        println!("  -> Ask Quantities 1-10: {:?}", ask_vols);
                        println!("  -> Bid Prices 1-10: {:?}", bid_prices);
                        println!("  -> Bid Quantities 1-10: {:?}", bid_vols);
                        println!("--------------------------------------------------");
                    }
                } else {
                    println!("Not Order Book Data: {}", text);
                }
            }
            Ok(Message::Ping(ping)) => {
                // 서버에서 연결 유지를 위해 Ping을 보내면 Pong으로 응답
                write.send(Message::Pong(ping)).await?;
            }
            Err(e) => {
                println!("WebSocket Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
