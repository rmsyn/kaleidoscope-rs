#[macro_export]
macro_rules! M {
    ($k:tt) => {
        $crate::key_defs::Key::from_raw($crate::plugins::ranges::MACRO_FIRST + $k as u16)
    };
}
