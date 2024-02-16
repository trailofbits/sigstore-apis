macro_rules! include_service {
    ($name:ident) => {
        #[allow(warnings)]
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
