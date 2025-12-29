use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;

#[derive(Deserialize)]
struct GitHubLabel {
    name: String,
    color: String,
}

#[derive(Deserialize)]
struct GitHubIssue {
    title: String,
    html_url: String,
    number: u32,
    state: String,
    labels: Vec<GitHubLabel>,
}

pub fn fetch_github_issue(owner: &str, repo: &str, num: &str, token: Option<&str>) -> String {
    let client = reqwest::blocking::Client::new();
    let url = format!(
        "https://api.github.com/repos/{}/{}/issues/{}",
        owner, repo, num
    );

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("mdbook-preprocessor"));

    if let Some(t) = token {
        if let Ok(auth_value) = HeaderValue::from_str(&format!("token {}", t)) {
            headers.insert(AUTHORIZATION, auth_value);
        }
    } else {
        eprintln!("Github token was not specified, falling back to regular link");
        return format!("https://github.com/{}/{}/issues/{}", owner, repo, num);
    }

    match client.get(&url).headers(headers).send() {
        Ok(response) if response.status().is_success() => {
            if let Ok(issue) = response.json::<GitHubIssue>() {
                return format_issue(issue, owner, repo);
            }
        }
        _ => {}
    }

    // Fallback: If API fails, just return a clickable raw link
    format!("https://github.com/{}/{}/issues/{}", owner, repo, num)
}

fn format_issue(issue: GitHubIssue, owner: &str, repo: &str) -> String {
    let status_icon = if issue.state == "open" { "⊙" } else { "✓" };
    let status_class = if issue.state == "open" {
        "gh-status-open"
    } else {
        "gh-status-closed"
    };

    let labels_html: String = issue
        .labels
        .iter()
        .map(|l| {
            format!(
                r#"<span class="gh-label" style="border-color:#{c};color:#{c}">{n}</span>"#,
                c = l.color,
                n = l.name
            )
        })
        .collect();

    format!(
        r#"<a href="{url}" target="_blank" class="gh-issue-card"><div class="gh-header"><span class="{status_class}">{status_icon}</span><span class="gh-title-text">{title}</span></div><div class="gh-labels">{labels}</div><div class="gh-meta">{owner}/{repo} <span style="opacity:0.5">#{number}</span></div></a>"#,
        url = issue.html_url,
        status_class = status_class,
        status_icon = status_icon,
        title = issue.title,
        labels = labels_html,
        owner = owner,
        repo = repo,
        number = issue.number,
    )
}
