use tonic::Status;

#[derive(Debug)]
pub struct ServError(pub rsys::error::RsysError);

impl From<ServError> for tonic::Status {
    fn from(value: ServError) -> Self {
        let err = value.0.to_string();
        Status::invalid_argument(err)
    }
}

impl From<ServError> for anyhow::Error {
    fn from(value: ServError) -> Self {
        let err = value.0.to_string();
        anyhow::anyhow!(err)
    }
}
