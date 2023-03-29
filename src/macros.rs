#[macro_export]
macro_rules! return_on_err {
    ($errfn:expr) => {
        if let Ok(val) = $errfn {
            val
        } else {
            return;
        }
    };
}
