use std::{
    collections::HashSet,
    fmt::{self, Display, Formatter},
};

use scraper::{Html, Selector};

pub struct Page {
    pub url: AbsoluteUrl,
    pub html: Html,
}

impl Page {
    pub async fn new(url: AbsoluteUrl) -> anyhow::Result<Page> {
        let text = Page::extract_html(&url.full_url()).await?;
        let html = Html::parse_document(&text);

        Ok(Page {
            url: url,
            html: html,
        })
    }

    async fn extract_html(url: &str) -> anyhow::Result<String> {
        let response = reqwest::get(url).await?;
        let raw = response.text().await?;
        Ok(raw)
    }

    pub async fn extract_links(&self) -> anyhow::Result<HashSet<AbsoluteUrl>> {
        let selector =
            Selector::parse("a[href]").map_err(|e| anyhow::anyhow!("Invalid selector: {:?}", e))?;
        let mut unique_links: HashSet<AbsoluteUrl> = HashSet::new();

        for element in self.html.select(&selector) {
            if let Some(href) = element.value().attr("href")
                && href.ends_with(".html")
            {
                unique_links.insert(AbsoluteUrl::new(&self.url.base, Some(href)));
            }
        }

        Ok(unique_links)
    }
}

#[derive(PartialEq, Eq, Hash)]
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

    pub fn full_url(&self) -> String {
        let mut full = self.base.clone();
        match &self.relative {
            Some(value) => {
                full.push('/');
                full.push_str(&value);
                return full;
            }
            None => return full,
        }
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
        let metadata = AbsoluteUrl::new("google.com", Some("search"));

        // act
        let full_url = metadata.full_url();

        // assert
        assert_eq!(
            "google.com/search".to_string(),
            full_url,
            "Full Url should contain a backslash"
        )
    }

    #[test]
    fn test_full_url_with_no_relative_path() {
        // arrange
        let metadata = AbsoluteUrl::new("google.com", None);

        // act
        let full_url = metadata.full_url();

        // assert
        assert_eq!(
            "google.com".to_string(),
            full_url,
            "Full Url should contain a backslash"
        )
    }
}
