macro_rules! include_service {
    ($name:ident) => {
        #[allow(warnings)]
        #[cfg(not(doctest))]
        pub mod $name {
            include!(concat!(
                env!("OUT_DIR"),
                "/codegen_",
                stringify!($name),
                ".rs"
            ));
        }
    };
}

#[cfg(not(doctest))]
pub mod types {
    use crate::rekor::types as rekor;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(tag = "type")]
    pub enum ProposedEntry {
        Rekord(rekor::Rekord),
        Hashedrekord(rekor::Hashedrekord),
        Rpm(rekor::Rpm),
        Tuf(rekor::Tuf),
        Helm(rekor::Helm),
        Intoto(rekor::Intoto),
        Cose(rekor::Cose),
        Jar(rekor::Jar),
        Rfc3161(rekor::Rfc3161),
        Dsse(rekor::Dsse),
    }
}

include_service!(fulcio);
include_service!(rekor);

/// Basic online tests to ensure that the generated client isn't completely broken.
#[cfg(test)]
mod tests {
    use crate::{
        fulcio,
        rekor::{self, types::SearchLogQuery},
    };

    const REKOR_URL: &str = "https://rekor.sigstore.dev";
    const FULCIO_URL: &str = "https://fulcio.sigstore.dev";

    #[tokio::test]
    async fn rekor_get_log_info() {
        let client = rekor::Client::new(REKOR_URL);

        let response = client.get_log_info(None).await;
        assert!(response.is_ok(), "{:?}", response.unwrap_err());
    }

    #[tokio::test]
    async fn fulcio_get_configuration() {
        let client = fulcio::Client::new(FULCIO_URL);

        let response = client.ca_get_configuration().await;
        assert!(response.is_ok(), "{:?}", response.unwrap_err());
    }

    #[tokio::test]
    async fn rekor_get_log_entry() {
        let client = rekor::Client::new(REKOR_URL);

        // Silly but almost certainly safe assumption: the log probably has at
        // least one entry.
        let response = client.get_log_entry_by_index(Some(0)).await;
        assert!(response.is_ok(), "{:?}", response.unwrap_err());
    }

    #[tokio::test]
    async fn rekor_search_log_entries_by_index() {
        let client = rekor::Client::new(REKOR_URL);

        let query = SearchLogQuery {
            entries: vec![],
            entry_uui_ds: vec![],
            log_indexes: vec![0, 1, 2],
        };

        let response = client.search_log_query(&query).await;
        assert!(response.is_ok(), "{:?}", response.unwrap_err());
        assert_eq!(response.unwrap().len(), 3);
    }
}
