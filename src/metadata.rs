//! Author metadata definitions
use chrono::Datelike;
use serde::Deserialize;

/// A Creative Commons license
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(Deserialize)]
pub enum License {
    #[serde(rename = "cc0")]
    PublicDomain,
    #[serde(rename = "cc-by")]
    Attribution,
    #[serde(rename = "cc-by-sa")]
    AttributionSa,
    #[serde(rename = "cc-by-nd")]
    AttributionNd,
    #[serde(rename = "cc-by-nc")]
    AttributionNc,
    #[serde(rename = "cc-by-nc-sa")]
    AttributionNcSa,
    #[serde(rename = "cc-by-nc-nd")]
    AttributionNcNd,
}

impl License {
    /// The canonical name of this license
    pub fn name(&self) -> &'static str {
        match self {
            Self::PublicDomain => "CC0 1.0 Universal",
            Self::Attribution => "Creative Commons Attribution 4.0 International",
            Self::AttributionSa => "Creative Commons Attribution-ShareAlike 4.0 International",
            Self::AttributionNd => "Creative Commons Attribution-NoDerivatives 4.0 International",
            Self::AttributionNc => "Creative Commons Attribution-NonCommercial 4.0 International",
            Self::AttributionNcSa => {
                "Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International"
            }
            Self::AttributionNcNd => {
                "Creative Commons Attribution-NonCommercial-NoDerivatives 4.0 International"
            }
        }
    }

    /// The official URL of this license
    pub fn url(&self) -> &'static str {
        match self {
            Self::PublicDomain => "https://creativecommons.org/publicdomain/zero/1.0/",
            Self::Attribution => "https://creativecommons.org/licenses/by/4.0/",
            Self::AttributionSa => "https://creativecommons.org/licenses/by-sa/4.0/",
            Self::AttributionNd => "https://creativecommons.org/licenses/by-nd/4.0/",
            Self::AttributionNc => "https://creativecommons.org/licenses/by-nc/4.0/",
            Self::AttributionNcSa => "https://creativecommons.org/licenses/by-nc-sa/4.0/",
            Self::AttributionNcNd => "https://creativecommons.org/licenses/by-nc-nd/4.0/",
        }
    }
}

/// An author
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
#[derive(Deserialize)]
pub struct Author {
    pub name: String,
    pub url: Option<String>,
}

// A full set of metadata
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
#[derive(Deserialize)]
pub struct Metadata {
    pub author: Author,
    pub license: Option<License>,
}

impl Metadata {
    /// The copyright notice corresponding to this metadata
    pub fn copyright(&self, date: impl Datelike) -> String {
        let author = &self.author.name;
        let (_, year) = date.year_ce();
        match self.license {
            Some(License::PublicDomain) => format!("© {author}, {year}. No rights reserved."),
            Some(_) => format!("© {author}, {year}. Some rights reserved."),
            None => format!("© {author}, {year}. All rights reserved."),
        }
    }

    /// The full license text corresponding to this metadata
    pub fn usage_terms(&self) -> Option<String> {
        self.license.as_ref().map(|l| match l {
            License::PublicDomain => {
                let author = &self.author.name;
                format!("To the extent possible under law, {author} has waived all copyright and related or neighboring rights to this work.")
            }
            _ => {
                let name = l.name();
                let url = l.url();
                format!("This work is licensed under the {name} License. To view a copy of this license, visit {url} or send a letter to Creative Commons, 171 Second Street, Suite 300, San Francisco, California, 94105, USA.")
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use toml::{de::Error, from_str};

    #[test]
    fn minimal_document() -> Result<(), Error> {
        let input = from_str::<Metadata>(
            r#"
            author.name = "Simon Sigurdhsson"
            "#,
        )?;
        let expected = Metadata {
            author: Author {
                name: "Simon Sigurdhsson".into(),
                url: None,
            },
            license: None,
        };
        assert_eq!(input, expected);
        Ok(())
    }

    #[test]
    fn full_document() -> Result<(), Error> {
        let input = from_str::<Metadata>(
            r#"
            author.name = "Simon Sigurdhsson"
            author.url = "http://photography.sigurdhsson.org/"
            license = "cc-by-nc"
            "#,
        )?;
        let expected = Metadata {
            author: Author {
                name: "Simon Sigurdhsson".into(),
                url: Some("http://photography.sigurdhsson.org/".into()),
            },
            license: Some(License::AttributionNc),
        };
        assert_eq!(input, expected);
        Ok(())
    }

    #[test]
    fn license_texts() -> Result<(), Error> {
        let no_license = Metadata {
            author: Author {
                name: "Simon Sigurdhsson".into(),
                url: None,
            },
            license: None,
        };
        assert_eq!(no_license.license.as_ref().map(|l| l.url()), None);
        assert_eq!(
            no_license.copyright(NaiveDate::from_yo(2025, 1)),
            "© Simon Sigurdhsson, 2025. All rights reserved."
        );
        assert_eq!(no_license.usage_terms(), None);

        let public_domain = Metadata {
            license: Some(License::PublicDomain),
            ..no_license.clone()
        };
        assert_eq!(
            public_domain.license.as_ref().map(|l| l.url()),
            Some("https://creativecommons.org/publicdomain/zero/1.0/")
        );
        assert_eq!(
            public_domain.copyright(NaiveDate::from_yo(2025, 1)),
            "© Simon Sigurdhsson, 2025. No rights reserved."
        );
        assert_eq!(
            public_domain.usage_terms(),
            Some("To the extent possible under law, Simon Sigurdhsson has waived all copyright and related or neighboring rights to this work.".into())
        );

        let cc_by_nc = Metadata {
            license: Some(License::AttributionNc),
            ..no_license.clone()
        };
        assert_eq!(
            cc_by_nc.license.as_ref().map(|l| l.url()),
            Some("https://creativecommons.org/licenses/by-nc/4.0/")
        );
        assert_eq!(
            cc_by_nc.copyright(NaiveDate::from_yo(2025, 1)),
            "© Simon Sigurdhsson, 2025. Some rights reserved."
        );
        assert_eq!(
            cc_by_nc.usage_terms(),
            Some("This work is licensed under the Creative Commons Attribution-NonCommercial 4.0 International License. To view a copy of this license, visit https://creativecommons.org/licenses/by-nc/4.0/ or send a letter to Creative Commons, 171 Second Street, Suite 300, San Francisco, California, 94105, USA.".into())
        );

        Ok(())
    }
}
