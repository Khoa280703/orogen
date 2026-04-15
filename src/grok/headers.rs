use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use rand::Rng;

const GROK_ORIGIN: &str = "https://grok.com";
const CHROME_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36";

/// Build headers matching Chrome 136 for Grok API requests.
/// Order matches grok2api-rs exactly.
pub fn build_grok_headers(
    cookie_header: &str,
    cf_clearance: Option<&str>,
) -> Vec<(&'static str, String)> {
    // Append cf_clearance to cookie if available
    let cookie = match cf_clearance {
        Some(cf) if !cf.is_empty() => format!("{cookie_header};cf_clearance={cf}"),
        _ => cookie_header.to_string(),
    };

    vec![
        ("Accept", "*/*".into()),
        ("Accept-Encoding", "gzip, deflate, br, zstd".into()),
        ("Accept-Language", "zh-CN,zh;q=0.9".into()),
        (
            "Baggage",
            "sentry-environment=production,sentry-release=d6add6fb0460641fd482d767a335ef72b9b6abb8,sentry-public_key=b311e0f2690c81f25e2c4cf6d4f7ce1c".into(),
        ),
        ("Cache-Control", "no-cache".into()),
        ("Content-Type", "application/json".into()),
        ("Cookie", cookie),
        ("Origin", GROK_ORIGIN.into()),
        ("Pragma", "no-cache".into()),
        ("Priority", "u=1, i".into()),
        ("Referer", format!("{GROK_ORIGIN}/")),
        (
            "Sec-Ch-Ua",
            r#""Google Chrome";v="136", "Chromium";v="136", "Not(A:Brand";v="24""#.into(),
        ),
        ("Sec-Ch-Ua-Arch", "arm".into()),
        ("Sec-Ch-Ua-Bitness", "64".into()),
        ("Sec-Ch-Ua-Mobile", "?0".into()),
        ("Sec-Ch-Ua-Model", String::new()),
        ("Sec-Ch-Ua-Platform", r#""macOS""#.into()),
        ("Sec-Fetch-Dest", "empty".into()),
        ("Sec-Fetch-Mode", "cors".into()),
        ("Sec-Fetch-Site", "same-origin".into()),
        ("User-Agent", CHROME_UA.into()),
        ("x-statsig-id", generate_statsig_id()),
        ("x-xai-request-id", uuid::Uuid::new_v4().to_string()),
    ]
}

/// Generate a random x-statsig-id (base64-encoded fake JS error, mimics Grok frontend telemetry)
fn generate_statsig_id() -> String {
    let mut rng = rand::rng();
    let msg = if rng.random_bool(0.5) {
        let suffix: String = (0..5)
            .map(|_| rng.random_range(b'a'..=b'z') as char)
            .collect();
        format!("e:TypeError: Cannot read properties of null (reading 'children[\"{suffix}\"]')")
    } else {
        let suffix: String = (0..10)
            .map(|_| rng.random_range(b'a'..=b'z') as char)
            .collect();
        format!("e:TypeError: Cannot read properties of undefined (reading '{suffix}')")
    };
    BASE64.encode(msg.as_bytes())
}
