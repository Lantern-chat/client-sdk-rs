use bytes::BytesMut;
use smol_str::SmolStr;
use tokio::io::{AsyncRead, AsyncReadExt};

use super::{Client, ClientError};
use crate::{
    api::commands::file::{CreateFile, CreateFileBody},
    models::Snowflake,
};

impl Client {
    /// Upload a plain file from its handle
    ///
    /// This does not do any extra handling for media files,
    /// such as finding dimensions or generating previews.
    #[cfg(feature = "fs")]
    pub async fn upload_plain_file(
        &self,
        filename: impl Into<SmolStr>,
        mime: Option<mime::Mime>,
        file: &mut tokio::fs::File,
        progress: impl FnMut(u64, u64),
    ) -> Result<Snowflake, ClientError> {
        let meta = file.metadata().await?;

        if !meta.is_file() {
            return Err(ClientError::NotAFile);
        }

        let meta = CreateFileBody {
            filename: filename.into(),
            size: match i32::try_from(meta.len()) {
                Ok(size) => size,
                Err(_) => return Err(ClientError::FileTooLarge),
            },
            width: None,
            height: None,
            mime: mime.map(SmolStr::from),
            preview: None,
        };

        self.upload_stream(meta, file, progress).await
    }

    /// Uploads a file stream in chunks
    pub async fn upload_stream(
        &self,
        meta: CreateFileBody,
        stream: impl AsyncRead,
        mut progress: impl FnMut(u64, u64),
    ) -> Result<Snowflake, ClientError> {
        let file_size = meta.size as u64;
        let file_id = self.driver().execute(CreateFile { body: meta }).await?;

        // TODO: Retrieve chunk size from server? Or set it from Client?
        const CHUNK_SIZE: usize = 1024 * 1024 * 8; // 8MiB

        let mut buffer = BytesMut::new();
        let mut read = 0;

        let mut stream = std::pin::pin!(stream);

        loop {
            // keep the buffer topped up at CHUNK_SIZE
            buffer.reserve(CHUNK_SIZE - buffer.capacity());

            // fill buffer
            while buffer.len() < buffer.capacity() {
                if 0 == stream.read_buf(&mut buffer).await? {
                    break;
                }
            }

            if buffer.len() == 0 {
                break;
            }

            let offset = read;

            read += buffer.len() as u64;

            let new_offset = self
                .driver()
                .patch_file(file_id, offset, buffer.split().freeze().into())
                .await?;

            if new_offset != read {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Upload request returned unexpected offset",
                )
                .into());
            }

            progress(read, file_size);
        }

        if file_size != read {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "File stream terminated too early",
            )
            .into());
        }

        Ok(file_id)
    }
}
