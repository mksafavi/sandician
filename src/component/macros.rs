#[cfg(test)]
macro_rules! assert_color_srgb_eq {
    ($a:expr,$b:expr) => {
        assert_color_srgb_eq!($a, $b, 0.01);
    };
    ($a:expr,$b:expr, $threshold:expr) => {
        if ($a.to_srgba().red - $b.to_srgba().red).abs() >= $threshold
            || ($a.to_srgba().green - $b.to_srgba().green).abs() >= $threshold
            || ($a.to_srgba().blue - $b.to_srgba().blue).abs() >= $threshold
        {
            panic!(
                "assertion `left == right` failed left:{:?} left{:?}",
                $a, $b
            );
        }
    };
}

#[cfg(test)]
pub(crate) use assert_color_srgb_eq;
