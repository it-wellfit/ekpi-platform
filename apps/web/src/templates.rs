// Askama templates for EKPI web

use askama::Template;

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomeTemplate {
    pub title: String,
    pub csrf_token: String,
}
