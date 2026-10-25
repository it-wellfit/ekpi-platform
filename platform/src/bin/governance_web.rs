#![forbid(unsafe_code)]

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use ekp_platform::governance::{default_ledger_path, Ledger};
use ekp_platform::governance_web;

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

    let app = governance_web::router(Arc::new(Mutex::new(ledger)));
    let address = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = match tokio::net::TcpListener::bind(address).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("無法監聽 {address}: {error}");
            std::process::exit(1);
        }
    };
    println!("治理工作台已在 http://{address} 啟動；按 Ctrl+C 停止。");
    if let Err(error) = axum::serve(listener, app).await {
        eprintln!("治理工作台停止：{error}");
        std::process::exit(1);
    }
}
