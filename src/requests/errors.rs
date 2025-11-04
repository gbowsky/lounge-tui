pub enum ErrorCode {
    IbiServersDown,
    IbiBadResponse,
    FailedToRetrieve,
    DataMismatchError,
    FailedToParseSchedulesRows,
    FailedToParseSchedulesDate,
}

impl ErrorCode {
    pub fn get_description(&self) -> String {
        match self {
            ErrorCode::IbiServersDown => String::from("IBI servers down"),
            ErrorCode::IbiBadResponse => String::from("Malformed response from IBI"),
            ErrorCode::FailedToRetrieve => String::from("Request failed on our side"),
            ErrorCode::DataMismatchError => String::from("Data mismatched on client's side"),
            ErrorCode::FailedToParseSchedulesRows => String::from("Failed to parse schedules (rows)"),
            ErrorCode::FailedToParseSchedulesDate => String::from("Failed to parse schedules (dates)"),
        }
    }
}