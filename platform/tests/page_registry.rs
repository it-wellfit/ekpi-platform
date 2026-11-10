// S11 頁面註冊表覆蓋鎖：走訪 PAGE_REGISTRY 逐頁驗證——
// requires_auth 頁未登入 → 401；登入後 → 200；全頁零 <script>。

use axum::body::Body;
use axum::http::{Request, StatusCode};
use ekp_platform::governance::Ledger;
use ekp_platform::governance_web::{self, PAGE_REGISTRY};
use ekp_platform::local_auth::AuthStore;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tower::ServiceExt;

fn app() -> axum::Router {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "ekpi-registry-{}-{}.jsonl",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let ledger = Arc::new(std::sync::Mutex::new(Ledger::open(&path).unwrap()));
    // This is a route-contract fixture, not a password-strength test: the
    // production default (600,000 PBKDF2 iterations) is covered by
    // local_auth unit tests. A low fixture cost keeps the parallel suite
    // inside the 15-second request timeout.
    let mut store = AuthStore::with_iterations(3600, 1_000);
    store
        .register(
            "tester",
            "correct-horse-battery-staple",
            ekp_platform::local_auth::Role::Approver,
        )
        .unwrap();
    let auth = Arc::new(std::sync::Mutex::new(store));
    let approvals = Arc::new(std::sync::Mutex::new(
        ekp_adapters::jsonl_approval::ApprovalStore::ephemeral().unwrap(),
    ));
    governance_web::router_with_approvals(ledger, auth, approvals)
}

async fn login(app: &axum::Router) -> String {
    let page = app
        .clone()
        .oneshot(
            Request::get("/login")
                .header("host", "127.0.0.1:8080")
                .header("x-forwarded-for", "10.9.0.1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(page.status(), StatusCode::OK);
    let csrf_cookie = page
        .headers()
        .get("set-cookie")
        .expect("login page sets csrf cookie")
        .to_str()
        .unwrap()
        .to_string();
    let token = csrf_cookie
        .split(';')
        .next()
        .unwrap()
        .strip_prefix("ekpi_csrf=")
        .unwrap()
        .to_string();
    let body = format!("csrf_token={token}&username=tester&password=correct-horse-battery-staple");
    let response = app
        .clone()
        .oneshot(
            Request::post("/login")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("host", "127.0.0.1:8080")
                .header("x-forwarded-for", "10.9.0.1")
                .header("origin", "http://127.0.0.1:8080")
                .header("cookie", format!("ekpi_csrf={token}"))
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let session = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .find_map(|value| {
            let value = value.to_str().ok()?;
            value
                .starts_with("ekpi_session=")
                .then(|| value.split(';').next().unwrap().to_string())
        })
        .expect("session cookie");
    let rotated = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .find_map(|value| {
            let value = value.to_str().ok()?;
            value
                .starts_with("ekpi_csrf=")
                .then(|| value.split(';').next().unwrap().to_string())
        })
        .expect("rotated csrf cookie");
    format!("{session}; {rotated}")
}

#[test]
fn registry_has_unique_paths_titles_and_a_single_public_page() {
    let mut paths = std::collections::BTreeSet::new();
    let mut titles = std::collections::BTreeSet::new();
    for page in PAGE_REGISTRY {
        assert!(
            page.path.starts_with('/'),
            "registry path must be absolute: `{}`",
            page.path
        );
        assert!(
            paths.insert(page.path),
            "duplicate registry path: `{}`",
            page.path
        );
        assert!(
            titles.insert(page.title),
            "duplicate registry title: `{}`",
            page.title
        );
    }
    let public: Vec<&str> = PAGE_REGISTRY
        .iter()
        .filter(|page| !page.requires_auth)
        .map(|page| page.path)
        .collect();
    assert_eq!(
        public,
        vec!["/login"],
        "only the login page may be public; every other page must require a session"
    );
    assert!(
        PAGE_REGISTRY.iter().any(|page| page.in_nav),
        "the navigation shell needs at least one registry entry"
    );
}

#[tokio::test]
async fn registry_pages_enforce_auth_and_render_without_script() {
    let app = app();

    // 未登入：requires_auth 頁一律 401；public 頁（/login）200。
    // 每頁使用不同的 x-forwarded-for，避免 30 req/min 限流干擾頁面契約；
    // 限流行為由 web_health 的指標斷言與獨立的 429 測試覆蓋。
    for (index, page) in PAGE_REGISTRY.iter().enumerate() {
        let response = app
            .clone()
            .oneshot(
                Request::get(page.path)
                    .header("host", "127.0.0.1:8080")
                    .header("x-forwarded-for", format!("10.1.0.{}", index + 1))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let expected = if page.requires_auth {
            StatusCode::UNAUTHORIZED
        } else {
            StatusCode::OK
        };
        assert_eq!(
            response.status(),
            expected,
            "`{}` auth gate mismatch (registry requires_auth={})",
            page.path,
            page.requires_auth
        );
    }

    // 登入後：所有頁面 200 且零 <script>（/login 為入口本身，另行斷言）。
    let cookies = login(&app).await;
    for (index, page) in PAGE_REGISTRY
        .iter()
        .filter(|page| page.path != "/login")
        .enumerate()
    {
        let response = app
            .clone()
            .oneshot(
                Request::get(page.path)
                    .header("host", "127.0.0.1:8080")
                    .header("x-forwarded-for", format!("10.2.0.{}", index + 1))
                    .header("cookie", cookies.as_str())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "`{}` must render for an authenticated session",
            page.path
        );
        let body = axum::body::to_bytes(response.into_body(), 512 * 1024)
            .await
            .unwrap();
        let html = String::from_utf8_lossy(&body).to_string();
        assert!(
            !html.contains("<script"),
            "`{}` must stay zero-JS",
            page.path
        );
        assert!(
            html.contains(page.title),
            "`{}` must render its registered title `{}`",
            page.path,
            page.title
        );
        if page.in_nav {
            assert!(
                html.contains(&format!("\"{}\"", page.path)),
                "`{}` should be linked from the navigation shell",
                page.path
            );
        }
    }
}
