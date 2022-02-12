use tokio::io::{AsyncRead, AsyncReadExt};

use bytes::BytesMut;

use super::{Client, ClientError};
use crate::{
    api::commands::file::{CreateFile, CreateFileBody},
    models::Snowflake,
};

impl Client {
    /// Uploads a file in chunks
    pub async fn upload_file(
        &self,
        meta: CreateFileBody,
        file: impl AsyncRead,
        mut progress: impl FnMut(u64),
    ) -> Result<Snowflake, ClientError> {
        let file_size = meta.size as u64;
        let file_id = self.raw_driver().execute(CreateFile { body: meta }).await?;

        // TODO: Retrieve chunk size from server?
        let mut buffer = BytesMut::with_capacity(1024 * 1024 * 8); // 8MiB
        let mut read = 0;

        tokio::pin!(file);

        while 0 != file.read_buf(&mut buffer).await? {
            read += buffer.len() as u64;

            let mut crc32 = crc32fast::Hasher::new();
            crc32.update(&buffer);

            let offset = self
                .raw_driver()
                .patch_file(file_id, crc32.finalize(), read, buffer.split().freeze().into())
                .await?;

            if offset != read {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Upload request returned unexpected offset",
                )
                .into());
            }

            progress(read);
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
