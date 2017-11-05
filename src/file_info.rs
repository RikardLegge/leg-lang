#[derive(Debug)]
pub struct CodePoint {
    pub line_number_from: usize,
    pub line_number_to: usize,

    pub column_number_from: usize,
    pub column_number_to: usize
}

impl Clone for CodePoint {
    fn clone(&self) -> Self {
        return CodePoint {
            line_number_from: self.line_number_from,
            line_number_to: self.line_number_to,

            column_number_from: self.column_number_from,
            column_number_to: self.column_number_to,
        }
    }
}