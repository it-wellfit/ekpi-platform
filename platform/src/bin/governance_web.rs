#![forbid(unsafe_code)]

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use ekp_platform::governance::{Ledger, default_ledger_path};
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

    // There is deliberately no default account. A local operator must choose
    // the credentials explicitly; the password is never printed or persisted.
    match (
        std::env::var("EKPI_LOCAL_AUTH_USER"),
        std::env::var("EKPI_LOCAL_AUTH_PASSWORD"),
    ) {
        (Ok(username), Ok(password)) => {
            let role = std::env::var("EKPI_LOCAL_AUTH_ROLE")
                .ok()
                .and_then(|value| Role::parse_role(&value))
                .unwrap_or(Role::Editor);
            match auth_state
                .lock()
                .expect("auth store lock is not poisoned at startup")
                .register(&username, &password, role)
            {
                Ok(()) => println!(
                    "Local login enabled for user '{}' (role {}).",
                    username,
                    role.as_str()
                ),
                Err(error) => {
                    eprintln!("Cannot register EKPI_LOCAL_AUTH_USER: {error}");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            println!(
                "No local login configured. Set EKPI_LOCAL_AUTH_USER and EKPI_LOCAL_AUTH_PASSWORD before starting the workbench."
            );
        }
    }
    // The approval workflow store is a separate bounded context from the
    // governance ledger: own file, own domain-separated hash chain. Opening
    // it must succeed before serving; a tampered file fails startup loudly.
    let approval_path = ekp_adapters::jsonl_approval::default_approval_path();
    let approval_store = match ekp_adapters::jsonl_approval::ApprovalStore::open(&approval_path) {
        Ok(store) => store,
        Err(error) => {
            eprintln!(
                "無法開啟核准工作流資料 {}: {error}",
                approval_path.display()
            );
            std::process::exit(1);
        }
    };
    println!("核准工作流資料：{}", approval_path.display());
    let approvals_state = Arc::new(Mutex::new(approval_store));

    let app = governance_web::router_with_approvals(ledger_state, auth_state, approvals_state);
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
