use std::fmt;

/// Safe operational context; never contains broker URLs or event data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    SourceConnect,
    SinkConnect,
    SourceChannel,
    SinkChannel,
    ConfirmSelect,
    Receive,
    Publish,
    Confirm,
    Unroutable,
    Settle,
    Sanitize,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::SourceConnect => "source connection failed",
            Self::SinkConnect => "sink connection failed",
            Self::SourceChannel => "source channel creation failed",
            Self::SinkChannel => "sink channel creation failed",
            Self::ConfirmSelect => "enabling publisher confirms failed",
            Self::Receive => "source receive failed",
            Self::Publish => "sink publish failed",
            Self::Confirm => "sink publisher confirmation failed",
            Self::Unroutable => "sink publication was rejected or unroutable",
            Self::Settle => "source acknowledgement failed",
            Self::Sanitize => "event could not be sanitized",
        })
    }
}
impl std::error::Error for Error {}

#[allow(async_fn_in_trait)]
pub trait Source {
    type Delivery;
    async fn receive(&self) -> Result<Option<Self::Delivery>, Error>;
    fn body(delivery: &Self::Delivery) -> &[u8];
    async fn settle(delivery: Self::Delivery) -> Result<(), Error>;
}
#[allow(async_fn_in_trait)]
pub trait Sink {
    async fn publish(&self, body: &[u8]) -> Result<(), Error>;
}

/// Process at most one delivery. The source retains an unacknowledged delivery on failure.
pub async fn process_one<S: Source, T: Sink>(
    source: &S,
    sink: &T,
    path: &[&str],
) -> Result<bool, Error> {
    let Some(delivery) = source.receive().await? else {
        return Ok(false);
    };
    let body = crate::sanitization::redact(S::body(&delivery), path)
        .map_err(|_| Error::Sanitize)?;
    sink.publish(&body).await?;
    S::settle(delivery).await?;
    Ok(true)
}
