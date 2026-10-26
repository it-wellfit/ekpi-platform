#![forbid(unsafe_code)]

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use ekp_platform::governance::{default_ledger_path, Ledger};
use ekp_platform::governance_web;
use ekp_platform::local_auth::{AuthStore, Role};

#[tokio::main]
async fn main() {
    let path = default_ledger_path();
    let ledger = match Ledger::open(&path) {
        Ok(ledger) => ledger,
        Err(error) => {
            eprintln!("無法開啟治理稽核資料 {}: {error}", path.display());
            std::process::exit(1);
        }
    };
    println!("本機治理試點資料：{}", path.display());
    println!("安全提示：僅供本機非敏感資料試用，尚未連接企業身分或文件平台。");

    let ledger_state = Arc::new(Mutex::new(ledger));
    let auth_state = Arc::new(Mutex::new(AuthStore::new(3600)));

    // Seed a non-sensitive pilot user for the local trial.
    if let Err(e) = auth_state
        .lock()
        .unwrap()
        .register("pilot", "pilot123", Role::Editor)
    {
        eprintln!("種子使用者註冊（可能已存在）：{e}");
    }

    let app = governance_web::router(ledger_state, auth_state);
    let address = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = match tokio::net::TcpListener::bind(address).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("無法監聽 {address}: {error}");
            std::process::exit(1);
        }
    };
    println!("治理工作台已在 http://{address} 啟動；按 Ctrl+C 停止。");
    if let Err(error) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        eprintln!("治理工作台停止：{error}");
        std::process::exit(1);
    }
}

async fn shutdown_signal() {
    match tokio::signal::ctrl_c().await {
        Ok(()) => println!("收到停止訊號，正在完成進行中的請求。"),
        Err(error) => eprintln!("無法註冊停止訊號：{error}"),
    }
}
