pub struct InferError {
    pub message: String,
}

impl InferError {
    pub fn new(message: String) -> InferError {
        InferError { message }
    }
}
