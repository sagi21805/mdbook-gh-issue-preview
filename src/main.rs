use std::{io, process};

use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use regex::Regex;

mod issue;

pub struct GitHubIssuePreprocessor;

impl Preprocessor for GitHubIssuePreprocessor {
    fn name(&self) -> &str {
        "gh-issue-preview-preprocessor"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let re = Regex::new(r"https?://github\.com/([\w-]+)/([\w-]+)/issues/(\d+)").unwrap();
        let token = std::env::var("GITHUB_TOKEN").ok();

        book.for_each_mut(|item| {
            if let mdbook::book::BookItem::Chapter(chapter) = item {
                chapter.content = re
                    .replace_all(&chapter.content, |caps: &regex::Captures| {
                        let owner = &caps[1];
                        let repo = &caps[2];
                        let num = &caps[3];

                        // Call the API and return the Zed-style HTML string
                        issue::fetch_github_issue(owner, repo, num, token.as_deref())
                    })
                    .to_string();
            }
        });

        Ok(book)
    }
}

fn main() {
    let preprocessor = GitHubIssuePreprocessor;

    if std::env::args().len() > 1 {
        if std::env::args().nth(1).unwrap() == "supports" {
            process::exit(0);
        }
    }

    if let Err(e) = handle_preprocessing(&preprocessor) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn handle_preprocessing(pre: &impl Preprocessor) -> Result<(), mdbook::errors::Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;
    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;
    Ok(())
}
