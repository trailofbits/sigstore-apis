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

include_service!(fulcio);
include_service!(rekor);
