/**
 * Early return from a function with a given error
 * + in dev mode, print line called from in dev mode.
 */
#[macro_export]
macro_rules! err {
    ($e:expr) => {{
        // compute value to avoid ownership issues with debug_assertions
        let err = $e;
        #[cfg(debug_assertions)]
        {
            debug!("Returning error: {err:?}")
        }
        return Err(err);
    }};
}
