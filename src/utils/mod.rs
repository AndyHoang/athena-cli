pub mod display;
pub mod filter;

/// Creates styled header cells
#[macro_export]
macro_rules! athena_headers {
    ($($header:expr),* $(,)?) => {
        prettytable::Row::new(vec![
            $(prettytable::Cell::new($header).style_spec("Fb")),*
        ])
    };
}
