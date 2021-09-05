#[macro_export]
macro_rules! timer {
    ($str:expr, $target:expr) => {{
        let start = Instant::now();
        let result = $target;
        println!(
            "{} : Elapsed time = {}.{:03}",
            $str,
            start.elapsed().as_secs(),
            start.elapsed().subsec_nanos() / 1_000_000
        );
        result
    }};
}
