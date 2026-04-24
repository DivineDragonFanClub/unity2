const BACKEND_COUNT: usize = 0
    + cfg!(feature = "fe-engage") as usize
    ;

const _: () = assert!(
    BACKEND_COUNT == 1,
    "unity2: exactly one backend feature must be enabled. \
     Pick one of: `fe-engage` (default). \
     To add a new game target, see `unity2/src/backend_assertion.rs`."
);
