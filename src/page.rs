use regex::Regex;
use scraper::{Html, Selector};
use serde::Serialize;
use std::{
    collections::HashSet,
    fmt::{self, Display, Formatter},
};
use url::Url;

#[derive(Clone, Serialize)]
pub struct Page {
    pub url: AbsoluteUrl,
    pub html: String,
}

impl Page {
    pub async fn new(url: AbsoluteUrl) -> anyhow::Result<Page> {
        let absolute_url = url.full_url()?;
        let text = Page::extract_html(&absolute_url).await?;

        Ok(Page {
            url: url,
            html: text,
        })
    }

    async fn extract_html(url: &str) -> anyhow::Result<String> {
        let response = reqwest::get(url).await?;
        let raw = response.text().await?;
        Ok(raw)
    }

    pub async fn extract_links(&self) -> anyhow::Result<HashSet<AbsoluteUrl>> {
        let document = Html::parse_document(&self.html);
        let selector =
            Selector::parse("a[href]").map_err(|e| anyhow::anyhow!("Invalid selector: {:?}", e))?;
        let mut unique_links: HashSet<AbsoluteUrl> = HashSet::new();

        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href")
                && href.ends_with(".html")
                && !Page::contains_exxxx(href)
            {
                unique_links.insert(AbsoluteUrl::new(&self.url.base, Some(href)));
            }
        }

        Ok(unique_links)
    }

    fn contains_exxxx(s: &str) -> bool {
        let re = Regex::new(r"E\d{4}").unwrap();
        re.is_match(s)
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize)]
pub struct AbsoluteUrl {
    pub base: String,
    pub relative: Option<String>,
}

impl AbsoluteUrl {
    pub fn new(base: &str, relative: Option<&str>) -> Self {
        AbsoluteUrl {
            base: base.to_string(),
            relative: match relative {
                Some(value) => Some(value.to_string()),
                None => None,
            },
        }
    }

    pub fn full_url(&self) -> anyhow::Result<String> {
        let path: String = match &self.relative {
            Some(value) => AbsoluteUrl::combine_relative_url(&self.base, &value)?,
            None => self.base.clone(),
        };

        Ok(path)
    }

    fn combine_relative_url(base: &str, relative: &str) -> anyhow::Result<String> {
        // It was noted that one of 10 commandments of Moses was the following:
        // Thou shall not use this Url library anywhere else
        let parsed_base_url = Url::parse(base)?;
        let combined_url = parsed_base_url.join(relative)?;
        Ok(combined_url.to_string())
    }
}

impl Display for AbsoluteUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(Base: {}, Relative: {})",
            self.base,
            match &self.relative {
                Some(value) => value,
                None => "",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::page::AbsoluteUrl;

    #[test]
    fn test_full_url() {
        // arrange
        let metadata = AbsoluteUrl::new("https://google.com/", Some("search"));

        // act
        let full_url = metadata.full_url().unwrap();

        // assert
        assert_eq!(
            "https://google.com/search".to_string(),
            full_url,
            "Full Url should contain a backslash"
        )
    }

    #[test]
    fn test_full_url_without_backslash() {
        // arrange
        let metadata = AbsoluteUrl::new("https://google.com", Some("search"));

        // act
        let full_url = metadata.full_url().unwrap();

        // assert
        assert_eq!(
            "https://google.com/search".to_string(),
            full_url,
            "Full Url should contain a backslash"
        )
    }

    #[test]
    fn test_full_url_with_upward_relative_path() {
        // arrange
        let metadata = AbsoluteUrl::new(
            "https://google.com/search/settings",
            Some("../../index.html"),
        );

        // act
        let full_url = metadata.full_url().unwrap();

        // assert
        assert_eq!(
            "https://google.com/index.html".to_string(),
            full_url,
            "Full Url should contain a backslash"
        )
    }

    #[test]
    fn test_full_url_with_no_relative_path() {
        // arrange
        let metadata = AbsoluteUrl::new("google.com", None);

        // act
        let full_url = metadata.full_url().unwrap();

        // assert
        assert_eq!(
            "google.com".to_string(),
            full_url,
            "Full Url should contain a backslash"
        )
    }
}
