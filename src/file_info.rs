#[derive(Debug)]
pub struct CodePoint {
    pub index_from: usize,
    pub index_to: usize,

    pub line_number_from: usize,
    pub line_number_to: usize,

    pub column_number_from: usize,
    pub column_number_to: usize
}

impl CodePoint {
    pub fn new() -> CodePoint {
        return CodePoint {
            index_from: 0,
            index_to: 0,

            line_number_from: 0,
            line_number_to: 0,

            column_number_from: 0,
            column_number_to: 0
        }
    }
}
