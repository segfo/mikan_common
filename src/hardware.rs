mod frame_buffer;
pub use frame_buffer::{
    write_pixel_bgr, write_pixel_rgb, FrameBufferConfig, PixelFormat, PixelWriter,
};
pub mod x86_64;
pub use x86_64::io;
