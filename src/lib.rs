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
    #[serde(untagged, rename_all = "lowercase")]
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
        types::ProposedEntry,
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
            log_indexes: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        };

        let response = client.search_log_query(&query).await;
        assert!(response.is_ok(), "{:?}", response.unwrap_err());
        assert_eq!(response.unwrap().len(), 10);
    }

    #[test]
    fn test_proposed_entry_deserialize_hashedrekord() {
        let raw = r#"{"apiVersion":"0.0.1","kind":"hashedrekord","spec":{"data":{"hash":{"algorithm":"sha256","value":"135c891df9b15e7f79cc5173b46d0d1ec66dc889ff95e3ffd5cfd0bf669bef9f"}},"signature":{"content":"MEYCIQDUXfS70qVCqBo2oQIE1Rtkupe68OTCDPyfsAOXjY23mAIhAOrqQ4xP8mZj9g2lwvmYCOEtZR2vq8rAeDqdKnmHmPOb","publicKey":{"content":"LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0tCk1JSUcrRENDQm4rZ0F3SUJBZ0lVRnRPNDJHd3lRcW5TNU8valdtQTVxY01RTU1Fd0NnWUlLb1pJemowRUF3TXcKTnpFVk1CTUdBMVVFQ2hNTWMybG5jM1J2Y21VdVpHVjJNUjR3SEFZRFZRUURFeFZ6YVdkemRHOXlaUzFwYm5SbApjbTFsWkdsaGRHVXdIaGNOTWpRd05ERTVNREF5TlRReldoY05NalF3TkRFNU1EQXpOVFF6V2pBQU1Ga3dFd1lICktvWkl6ajBDQVFZSUtvWkl6ajBEQVFjRFFnQUV4elN2Zis4MXExamRraVBSelYvYTl0QW1ER2kyTi9sMkNiMVIKbHJsTW14U2ZwNjdzdGd5aWVFRFdTR0FpYy9RUEZpcWQvVkFOYzdwVCtqcG54cEZxV2FPQ0JaNHdnZ1dhTUE0RwpBMVVkRHdFQi93UUVBd0lIZ0RBVEJnTlZIU1VFRERBS0JnZ3JCZ0VGQlFjREF6QWRCZ05WSFE0RUZnUVV0UGZ4CmRCeUpTM3dGbXZ6cUgyaVpqdWNiRXdvd0h3WURWUjBqQkJnd0ZvQVUzOVBwejFZa0VaYjVxTmpwS0ZXaXhpNFkKWkQ4d2FBWURWUjBSQVFIL0JGNHdYSVphYUhSMGNITTZMeTluYVhSb2RXSXVZMjl0TDJOb1lXbHVaM1ZoY21RdAphVzFoWjJWekwybHRZV2RsY3k4dVoybDBhSFZpTDNkdmNtdG1iRzkzY3k5eVpXeGxZWE5sTG5saGJXeEFjbVZtCmN5OW9aV0ZrY3k5dFlXbHVNRGtHQ2lzR0FRUUJnNzh3QVFFRUsyaDBkSEJ6T2k4dmRHOXJaVzR1WVdOMGFXOXUKY3k1bmFYUm9kV0oxYzJWeVkyOXVkR1Z1ZEM1amIyMHdGZ1lLS3dZQkJBR0R2ekFCQWdRSWMyTm9aV1IxYkdVdwpOZ1lLS3dZQkJBR0R2ekFCQXdRb01XWTFZVGhrWlRJMU4yVmxNbU0yWTJVeE1USXpZakF5WmpBMFlXTXhaREF5Ck1qWTNNbUl3WXpBc0Jnb3JCZ0VFQVlPL01BRUVCQjR1WjJsMGFIVmlMM2R2Y210bWJHOTNjeTl5Wld4bFlYTmwKTG5saGJXd3dKZ1lLS3dZQkJBR0R2ekFCQlFRWVkyaGhhVzVuZFdGeVpDMXBiV0ZuWlhNdmFXMWhaMlZ6TUIwRwpDaXNHQVFRQmc3OHdBUVlFRDNKbFpuTXZhR1ZoWkhNdmJXRnBiakE3QmdvckJnRUVBWU8vTUFFSUJDME1LMmgwCmRIQnpPaTh2ZEc5clpXNHVZV04wYVc5dWN5NW5hWFJvZFdKMWMyVnlZMjl1ZEdWdWRDNWpiMjB3YWdZS0t3WUIKQkFHRHZ6QUJDUVJjREZwb2RIUndjem92TDJkcGRHaDFZaTVqYjIwdlkyaGhhVzVuZFdGeVpDMXBiV0ZuWlhNdgphVzFoWjJWekx5NW5hWFJvZFdJdmQyOXlhMlpzYjNkekwzSmxiR1ZoYzJVdWVXRnRiRUJ5WldaekwyaGxZV1J6CkwyMWhhVzR3T0FZS0t3WUJCQUdEdnpBQkNnUXFEQ2d4WmpWaE9HUmxNalUzWldVeVl6WmpaVEV4TWpOaU1ESm0KTURSaFl6RmtNREl5TmpjeVlqQmpNQjBHQ2lzR0FRUUJnNzh3QVFzRUR3d05aMmwwYUhWaUxXaHZjM1JsWkRBNwpCZ29yQmdFRUFZTy9NQUVNQkMwTUsyaDBkSEJ6T2k4dloybDBhSFZpTG1OdmJTOWphR0ZwYm1kMVlYSmtMV2x0CllXZGxjeTlwYldGblpYTXdPQVlLS3dZQkJBR0R2ekFCRFFRcURDZ3haalZoT0dSbE1qVTNaV1V5WXpaalpURXgKTWpOaU1ESm1NRFJoWXpGa01ESXlOamN5WWpCak1COEdDaXNHQVFRQmc3OHdBUTRFRVF3UGNtVm1jeTlvWldGawpjeTl0WVdsdU1Ca0dDaXNHQVFRQmc3OHdBUThFQ3d3Sk5UWXpOVEV3T1RVeU1EUUdDaXNHQVFRQmc3OHdBUkFFCkpnd2thSFIwY0hNNkx5OW5hWFJvZFdJdVkyOXRMMk5vWVdsdVozVmhjbVF0YVcxaFoyVnpNQmtHQ2lzR0FRUUIKZzc4d0FSRUVDd3dKTVRFek1UazROVFExTUdvR0Npc0dBUVFCZzc4d0FSSUVYQXhhYUhSMGNITTZMeTluYVhSbwpkV0l1WTI5dEwyTm9ZV2x1WjNWaGNtUXRhVzFoWjJWekwybHRZV2RsY3k4dVoybDBhSFZpTDNkdmNtdG1iRzkzCmN5OXlaV3hsWVhObExubGhiV3hBY21WbWN5OW9aV0ZrY3k5dFlXbHVNRGdHQ2lzR0FRUUJnNzh3QVJNRUtnd28KTVdZMVlUaGtaVEkxTjJWbE1tTTJZMlV4TVRJellqQXlaakEwWVdNeFpEQXlNalkzTW1Jd1l6QVlCZ29yQmdFRQpBWU8vTUFFVUJBb01DSE5qYUdWa2RXeGxNRjRHQ2lzR0FRUUJnNzh3QVJVRVVBeE9hSFIwY0hNNkx5OW5hWFJvCmRXSXVZMjl0TDJOb1lXbHVaM1ZoY21RdGFXMWhaMlZ6TDJsdFlXZGxjeTloWTNScGIyNXpMM0oxYm5Ndk9EYzAKTmpNMU5qZzROQzloZEhSbGJYQjBjeTh4TUJZR0Npc0dBUVFCZzc4d0FSWUVDQXdHY0hWaWJHbGpNSUdLQmdvcgpCZ0VFQWRaNUFnUUNCSHdFZWdCNEFIWUEzVDB3YXNiSEVUSmpHUjRjbVdjM0FxSktYcmplUEszL2g0cHlnQzhwCjdvNEFBQUdPODc2dVdBQUFCQU1BUnpCRkFpRUFqT3VVWFBVZU90N2dyQjZRRFNPbUpqcFU3SGE1ckh1dGkrR3EKQndEc2lvNENJRjJzRDRjWG9zYk9HVUlNeHZtQmJKTW1acjJsdzZ1RnlHdW1BZURTcXNLVU1Bb0dDQ3FHU000OQpCQU1EQTJjQU1HUUNNRmV6TzR0bXRSTVdTM0hDeGw3U2R1QU8rQnZPWXVaZ2o3cExDU1g2ZlIvQysxWmZSMG1QCmxpMW5zNnRab2hGa3dRSXdNbGg0bWpLTWNpbWJiMmhIQzBRMDN3K1RUVW84cHpxQkdpdzJxM2F2QnZsMnFMV2EKODd6b1FXSmFaRHk1azF2TwotLS0tLUVORCBDRVJUSUZJQ0FURS0tLS0tCg=="}}}}"#;

        let entry = serde_json::from_str::<ProposedEntry>(raw).unwrap();
        assert!(matches!(entry, ProposedEntry::Hashedrekord(_)));
    }
}
