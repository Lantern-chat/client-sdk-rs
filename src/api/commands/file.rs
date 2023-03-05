use super::*;

command! {
    +struct CreateFile -> Snowflake: POST("file") {
        ;
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        struct CreateFileBody {
            pub filename: SmolStr,

            pub size: i32,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default, setter(into, strip_option)))]
            pub mime: Option<SmolStr>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default, setter(strip_option)))]
            pub width: Option<i32>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default, setter(strip_option)))]
            pub height: Option<i32>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub preview: Option<String>,
        }
    }

    +struct GetFilesystemStatus -> FilesystemStatus: OPTIONS("file") {}

    +struct GetFileStatus -> FileStatus: HEAD("file" / file_id) {
        pub file_id: Snowflake,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FilesystemStatus {
    pub quota_used: i64,
    pub quota_total: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FileStatus {
    pub complete: u32,
    pub upload_offset: u64,
}
