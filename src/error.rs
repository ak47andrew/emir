use image::ImageError;

use crate::window_manager::FontId;

#[derive(thiserror::Error, Debug)]
pub enum FontError {
    #[error("Failed to read font file at {path}: {source:?}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse font data: {reason}")]
    Parse { reason: String },
}

#[derive(thiserror::Error, Debug)]
pub enum WindowError {
    #[error("Unable to create window")]
    Create {
        #[source]
        source: minifb::Error,
    },

    #[error("Unable to update frame")]
    Update {
        #[source]
        source: minifb::Error,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum PixelBufferError {
    #[error(
        r#"Your "pixel buffer" doesn't fit, babe! I can only take {orig_w}x{orig_h}, but yours are {new_w}x{new_h}!"#
    )]
    IncorrectBufferSize {
        orig_w: usize,
        orig_h: usize,
        new_w: usize,
        new_h: usize,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum BufferError {
    #[error("Invalid address/out of bounds. Trying to access {x}x{y} on a {w}x{h} buffer")]
    InvalidAddress {
        x: usize,
        y: usize,
        w: usize,
        h: usize,
    },

    #[error(
        "Length of the itrator ({iterator_size}) mod w ({w}) should be 0. It's not. Sad. We don't know what to do. Send help. Please. I'm dying. Right now. I'm so tired of writing this errro messsage alone. Someone please stop me"
    )]
    InvalidSize { w: usize, iterator_size: usize },

    #[error(
        "Empty buffer provided. Can't get anything out of it. Don't know what to do. Please fix me :("
    )]
    EmptyIteratorOnInit {},
}

#[derive(thiserror::Error, Debug)]
pub enum TextureError {
    #[error("Failed to open texture at given path")]
    OpenError {
        path: Option<String>,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to decode a texture at given path")]
    DecodeError {
        path: Option<String>,
        #[source]
        source: ImageError,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum DrawError {
    #[error("Font with specified FontId is not loaded (possibly already unloaded)")]
    FontNotLoaded { font_id: FontId },

    #[error(
        "Function that draws to the buffer directly was called during render step. Use draw steps for that"
    )]
    DrawDuringRenderStep,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Font(#[from] FontError),

    #[error(transparent)]
    Window(#[from] WindowError),

    #[error(transparent)]
    PixelBuffer(#[from] PixelBufferError),

    #[error(transparent)]
    Buffer(#[from] BufferError),

    #[error(transparent)]
    Texture(#[from] TextureError),

    #[error(transparent)]
    Draw(#[from] DrawError),
}
