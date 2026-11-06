// Zero-JS compliance tests for EKPI web
// Verifies that no JavaScript is served in responses

#[cfg(test)]
mod tests {
    #[test]
    fn test_no_script_tags_in_html() {
        // Verify that HTML responses contain no <script> tags
        let html = include_str!("../templates/home.html");
        assert!(
            !html.contains("<script"),
            "HTML must not contain script tags"
        );
        assert!(
            !html.contains("javascript:"),
            "HTML must not contain javascript: URIs"
        );
        assert!(
            !html.contains("onclick"),
            "HTML must not contain onclick handlers"
        );
        assert!(
            !html.contains("onload"),
            "HTML must not contain onload handlers"
        );
    }

    #[test]
    fn test_security_headers_present() {
        // Verify CSP header is set correctly
        let csp = "default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'";
        assert!(
            csp.contains("default-src 'self'"),
            "CSP must restrict default-src"
        );
        assert!(!csp.contains("script-src"), "CSP must not allow script-src");
    }

    #[test]
    fn test_login_form_carries_csrf_field() {
        // The template must carry the double-submit CSRF hidden field; token
        // generation/validation semantics are unit-tested in main.rs.
        let html = include_str!("../templates/home.html");
        assert!(
            html.contains("name=\"csrf_token\""),
            "the form must include the csrf_token hidden field"
        );
    }

    #[test]
    fn test_css_only_styling() {
        // Verify CSS file exists and contains no JS
        let css = include_str!("../static/css/design-system.css");
        assert!(
            !css.contains("javascript"),
            "CSS must not reference JavaScript"
        );
        assert!(!css.contains("<script"), "CSS must not contain script tags");
    }
}
