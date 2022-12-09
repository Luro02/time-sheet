/// A macro to signal that some code is unreachable. In debug mode this will panic if
/// the code is reached for some reason, but in release it will cause undefined behaviour!
///
/// ### Safety
///
/// The macro call must never be reached, otherwise undefined behaviour will occur.
#[macro_export]
macro_rules! unreachable_unchecked {
    (@inner $($arg:tt)*) => {{
        #[cfg(debug_assertions)]
        {
            // ideally all arguments would be passed to the unreachable macro,
            // but that does expand to non-const code at the moment
            // ::core::unreachable!($($arg)*)

            ::core::panic!(concat!("internal error: entered unreachable code", $($arg)*));
        }
        #[cfg(not(debug_assertions))]
        unsafe {
            ::core::hint::unreachable_unchecked()
        }
    }};
    ($($e:expr)*) => {
        unreachable_unchecked!(@inner ": ", $($e)*)
    };
    () => {
        unreachable_unchecked!(@inner ".")
    };
}

#[macro_export]
macro_rules! min {
    ( $a:expr $(, $tail:expr)+ ) => ({
        // ::core::cmp::min($a, min!($($tail),*))
        let other = min!($($tail),+);
        if $a < other {
            $a
        } else {
            other
        }
    });
    ( $a:expr ) => ($a);
}

#[macro_export]
macro_rules! max {
    ( $a:expr $(, $tail:expr)+ ) => ({
        let other = max!($($tail),+);
        if $a > other {
            $a
        } else {
            other
        }
    });
    ( $a:expr ) => ($a);
}

#[macro_export]
macro_rules! iter_const {
    ( for $t:ident in $start:expr ,.. $end:expr => $bl:block ) => {{
        let mut $t = $start;
        if $start < $end {
            loop {
                $bl;

                $t += 1;
                if $t >= $end {
                    break;
                }
            }
        }
    }};
}

#[macro_export]
macro_rules! duration {
    () => {};
    ( $hours:literal : $mins:literal : $secs:literal ) => {{
        static_assertions::const_assert!($hours < 24);
        static_assertions::const_assert!($mins < 60);
        static_assertions::const_assert!($secs < 60);

        ::core::time::Duration::from_secs(
            ($hours as u64) * 60 * 60 + ($mins as u64) * 60 + ($secs as u64),
        )
    }};
}

#[macro_export]
macro_rules! map {
    ( $( $key:expr => $value:expr ),+ $(,)? ) => {
        {
            let mut _map = ::std::collections::HashMap::new();

            $(
                _map.insert($key, $value);
            )+

            _map
        }
    };
}
