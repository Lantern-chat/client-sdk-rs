use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum EmbedType {
    #[serde(alias = "image")]
    Img,
    Audio,
    #[serde(alias = "video")]
    Vid,
    Html,
    Link,
    Article,
}

bitflags::bitflags! {
    pub struct EmbedFlags: u8 {
        /// This embed contains spoilered content and should be displayed as such
        const SPOILER   = 1 << 0;

        /// This embed may contain content marked as "adult"
        ///
        /// NOTE: This is not always accurate, and is provided on a best-effort basis
        const ADULT     = 1 << 1;
    }
}

serde_shims::impl_serde_for_bitflags!(EmbedFlags);
impl_schema_for_bitflags!(EmbedFlags);

fn is_none_or_empty(value: &Option<SmolStr>) -> bool {
    match value {
        Some(ref value) => value.is_empty(),
        None => true,
    }
}

pub type MaybeEmbedMedia = Option<Box<EmbedMedia>>;

/// An embed is metadata taken from a given URL by loading said URL, parsing any meta tags, and fetching
/// extra information from oEmbed sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EmbedV1 {
    /// Timestamp when the embed was retreived
    pub ts: Timestamp,

    /// Embed type
    #[serde(alias = "type")]
    pub ty: EmbedType,

    #[serde(
        rename = "f",
        alias = "flags",
        default = "EmbedFlags::empty",
        skip_serializing_if = "EmbedFlags::is_empty"
    )]
    pub flags: EmbedFlags,

    /// URL fetched
    #[serde(rename = "u", alias = "url", default, skip_serializing_if = "is_none_or_empty")]
    pub url: Option<SmolStr>,

    /// Canonical URL
    #[serde(rename = "c", alias = "canonical", default, skip_serializing_if = "is_none_or_empty")]
    pub canonical: Option<SmolStr>,

    #[serde(rename = "t", alias = "title", default, skip_serializing_if = "is_none_or_empty")]
    pub title: Option<SmolStr>,

    /// Description, usually from the Open-Graph API
    #[serde(
        rename = "d",
        alias = "description",
        default,
        skip_serializing_if = "is_none_or_empty"
    )]
    pub description: Option<SmolStr>,

    /// Accent Color
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "color")]
    pub color: Option<u32>,

    #[serde(default, skip_serializing_if = "EmbedAuthor::is_none")]
    pub author: Option<EmbedAuthor>,

    /// oEmbed Provider
    #[serde(
        rename = "p",
        alias = "provider",
        default,
        skip_serializing_if = "EmbedProvider::is_none"
    )]
    pub provider: EmbedProvider,

    /// HTML and similar objects
    ///
    /// See: <https://www.html5rocks.com/en/tutorials/security/sandboxed-iframes/>
    #[serde(default, skip_serializing_if = "EmbedMedia::is_empty")]
    pub obj: MaybeEmbedMedia,
    #[serde(default, skip_serializing_if = "EmbedMedia::is_empty", alias = "image")]
    pub img: MaybeEmbedMedia,
    #[serde(default, skip_serializing_if = "EmbedMedia::is_empty")]
    pub audio: MaybeEmbedMedia,
    #[serde(
        rename = "vid",
        alias = "video",
        default,
        skip_serializing_if = "EmbedMedia::is_empty"
    )]
    pub video: MaybeEmbedMedia,
    #[serde(default, skip_serializing_if = "EmbedMedia::is_empty")]
    pub thumb: MaybeEmbedMedia,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<EmbedField>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub footer: Option<EmbedFooter>,
}

impl EmbedV1 {
    pub fn is_plain_link(&self) -> bool {
        if self.ty != EmbedType::Link
            || self.url.is_none()
            || !is_none_or_empty(&self.canonical)
            || !is_none_or_empty(&self.title)
            || !is_none_or_empty(&self.description)
            || self.color.is_some()
            || !EmbedAuthor::is_none(&self.author)
            || !EmbedProvider::is_none(&self.provider)
            || !EmbedMedia::is_empty(&self.obj)
            || !EmbedMedia::is_empty(&self.img)
            || !EmbedMedia::is_empty(&self.audio)
            || !EmbedMedia::is_empty(&self.video)
            || !EmbedMedia::is_empty(&self.thumb)
            || !self.fields.is_empty()
            || self.footer.is_some()
        {
            return false;
        }

        true
    }

    pub fn visit_text_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut SmolStr),
    {
        fn visit_text_opt_mut<F>(text: &mut Option<SmolStr>, mut f: F)
        where
            F: FnMut(&mut SmolStr),
        {
            if let Some(ref mut value) = *text {
                f(value);
            }
        }

        visit_text_opt_mut(&mut self.title, &mut f);
        visit_text_opt_mut(&mut self.description, &mut f);
        visit_text_opt_mut(&mut self.provider.name, &mut f);

        if let Some(ref mut author) = self.author {
            f(&mut author.name);
        }

        self.visit_media_mut(|media| visit_text_opt_mut(&mut media.alt, &mut f));

        for field in &mut self.fields {
            f(&mut field.name);
            f(&mut field.value);
        }

        if let Some(ref mut footer) = self.footer {
            f(&mut footer.text);
        }
    }

    /// Visit each [`EmbedMedia`] to mutate them (such as to generate the proxy signature)
    pub fn visit_media_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut EmbedMedia),
    {
        EmbedMedia::visit_mut(&mut self.obj, &mut f);
        EmbedMedia::visit_mut(&mut self.img, &mut f);
        EmbedMedia::visit_mut(&mut self.audio, &mut f);
        EmbedMedia::visit_mut(&mut self.video, &mut f);
        EmbedMedia::visit_mut(&mut self.thumb, &mut f);

        EmbedMedia::visit_mut(&mut self.provider.icon, &mut f);

        if let Some(ref mut footer) = self.footer {
            EmbedMedia::visit_mut(&mut footer.icon, &mut f);
        }

        if let Some(ref mut author) = self.author {
            EmbedMedia::visit_mut(&mut author.icon, &mut f);
        }

        for field in &mut self.fields {
            EmbedMedia::visit_mut(&mut field.img, &mut f);
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EmbedFooter {
    #[serde(rename = "t", alias = "text")]
    pub text: SmolStr,

    #[serde(rename = "i", alias = "icon", default, skip_serializing_if = "EmbedMedia::is_empty")]
    pub icon: MaybeEmbedMedia,
}

pub type UrlSignature = FixedStr<27>;

crate::util::fixed::impl_fixedstr_schema!(UrlSignature, "Base-64 encoded cryptographic signature");

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EmbedMedia {
    #[serde(rename = "u", alias = "url")]
    pub url: SmolStr,

    /// Non-visible description of the embedded media
    #[serde(default, skip_serializing_if = "is_none_or_empty")]
    pub alt: Option<SmolStr>,

    /// Cryptographic signature for use with the proxy server
    #[serde(rename = "s", alias = "signature", default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<FixedStr<27>>,

    /// height
    #[serde(rename = "h", alias = "height", default, skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,

    /// witdth
    #[serde(rename = "w", alias = "width", default, skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,

    #[serde(rename = "m", alias = "mime", default, skip_serializing_if = "is_none_or_empty")]
    pub mime: Option<SmolStr>,
}

impl EmbedMedia {
    pub fn is_empty(this: &MaybeEmbedMedia) -> bool {
        match this {
            Some(ref e) => e.url.is_empty(),
            None => true,
        }
    }

    pub fn visit_mut<F>(this: &mut MaybeEmbedMedia, mut f: F)
    where
        F: FnMut(&mut EmbedMedia),
    {
        if let Some(ref mut media) = this {
            f(&mut *media);
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EmbedProvider {
    #[serde(rename = "n", alias = "name", default, skip_serializing_if = "is_none_or_empty")]
    pub name: Option<SmolStr>,

    #[serde(rename = "u", alias = "url", default, skip_serializing_if = "is_none_or_empty")]
    pub url: Option<SmolStr>,

    #[serde(rename = "i", alias = "icon", default, skip_serializing_if = "EmbedMedia::is_empty")]
    pub icon: MaybeEmbedMedia,
}

impl EmbedProvider {
    pub fn is_none(&self) -> bool {
        is_none_or_empty(&self.name) && is_none_or_empty(&self.url) && EmbedMedia::is_empty(&self.icon)
    }
}

impl EmbedAuthor {
    pub fn is_none(this: &Option<Self>) -> bool {
        match this {
            Some(ref this) => {
                this.name.is_empty() && is_none_or_empty(&this.url) && EmbedMedia::is_empty(&this.icon)
            }
            None => true,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EmbedAuthor {
    #[serde(rename = "n", alias = "name")]
    pub name: SmolStr,

    #[serde(rename = "u", alias = "url", default, skip_serializing_if = "is_none_or_empty")]
    pub url: Option<SmolStr>,

    #[serde(rename = "i", alias = "icon", default, skip_serializing_if = "EmbedMedia::is_empty")]
    pub icon: MaybeEmbedMedia,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EmbedField {
    #[serde(rename = "n", alias = "name", default, skip_serializing_if = "SmolStr::is_empty")]
    pub name: SmolStr,
    #[serde(rename = "v", alias = "value", default, skip_serializing_if = "SmolStr::is_empty")]
    pub value: SmolStr,

    #[serde(default, skip_serializing_if = "EmbedMedia::is_empty", alias = "image")]
    pub img: MaybeEmbedMedia,

    /// Should use block-formatting
    #[serde(rename = "b", alias = "block", default, skip_serializing_if = "is_false")]
    pub block: bool,
}

impl EmbedField {
    pub fn is_empty(&self) -> bool {
        (self.name.is_empty() || self.value.is_empty()) && EmbedMedia::is_empty(&self.img)
    }
}

impl Default for EmbedV1 {
    #[inline]
    fn default() -> EmbedV1 {
        EmbedV1 {
            ts: Timestamp::UNIX_EPOCH,
            ty: EmbedType::Link,
            flags: EmbedFlags::empty(),
            url: None,
            canonical: None,
            title: None,
            description: None,
            color: None,
            author: None,
            provider: EmbedProvider::default(),
            img: None,
            audio: None,
            video: None,
            thumb: None,
            obj: None,
            fields: Vec::new(),
            footer: None,
        }
    }
}
