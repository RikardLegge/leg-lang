#[derive(Debug)]
pub struct CodePoint {
    pub line_number_from: usize,
    pub line_number_to: usize,

    pub column_number_from: usize,
    pub column_number_to: usize
}
