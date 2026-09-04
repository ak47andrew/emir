#[derive(thiserror::Error, Debug)]
pub enum FontError {
    #[error("Failed to read font file at {path}: {source:?}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse font data: {reason}")]
    Parse {
        reason: String,
    }
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
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PixelBufferError {
    #[error("Your \"pixel buffer\" doesn't fit, babe! I can only take {orig_w}x{orig_h}, but yours are {new_w}x{new_h}!")]
    IncorrectBufferSize {
        orig_w: usize,
        orig_h: usize,
        new_w: usize,
        new_h: usize,
    },

    #[error("Invalid address/out of bounds. Trying to access {x}x{y} on a {w}x{h} buffer")]
    InvalidAddress {
        x: usize,
        y: usize,
        w: usize,
        h: usize,
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Font(#[from] FontError),

    #[error(transparent)]
    Window(#[from] WindowError),

    #[error(transparent)]
    PixelBuffer(#[from] PixelBufferError),
}
