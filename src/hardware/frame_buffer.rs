pub type PixelWriter = unsafe fn(usize, usize, usize, *mut u8, usize, usize, [u8; 3]);
pub unsafe fn write_pixel_rgb(
    pixels_per_scan_line: usize,
    horizontal_resolution: usize,
    vertical_resolution: usize,
    pixel_base: *mut u8,
    x: usize,
    y: usize,
    rgb: [u8; 3],
) {
    let pixel_pos = x + y * pixels_per_scan_line;
    for idx in 0..3 {
        *(pixel_base.offset((pixel_pos * 4 + idx) as isize)) = rgb[idx];
    }
}
pub unsafe fn write_pixel_bgr(
    pixels_per_scan_line: usize,
    horizontal_resolution: usize,
    vertical_resolution: usize,
    pixel_base: *mut u8,
    x: usize,
    y: usize,
    rgb: [u8; 3],
) {
    let pixel_pos = x + y * pixels_per_scan_line;
    for idx in 0..3 {
        *(pixel_base.offset((pixel_pos * 4 + idx) as isize)) = rgb[2 - idx];
    }
}
#[derive(Debug, Clone, Copy)]
pub enum PixelFormat {
    RGBReserved8BitParColor,
    BGRReserved8BitParColor,
}
#[derive(Debug)]
pub struct FrameBufferConfig {
    base: *mut u8,
    pixels_per_scan_line: usize,
    horizontal_resolution: usize,
    vertical_resolution: usize,
    pixel_format: PixelFormat,
    writer: PixelWriter,
}
impl FrameBufferConfig {
    pub fn new(
        base: *mut u8,
        pixels_per_scan_line: usize,
        horizontal_resolution: usize,
        vertical_resolution: usize,
        pixel_format: PixelFormat,
    ) -> Self {
        FrameBufferConfig {
            base: base,
            pixels_per_scan_line: pixels_per_scan_line,
            horizontal_resolution: horizontal_resolution,
            vertical_resolution: vertical_resolution,
            pixel_format: pixel_format,
            writer: write_pixel_bgr,
        }
    }
    pub fn init(&mut self) {
        self.writer = match self.pixel_format {
            PixelFormat::RGBReserved8BitParColor => write_pixel_rgb,
            PixelFormat::BGRReserved8BitParColor => write_pixel_bgr,
        };
    }
    pub fn write_pixel(&mut self, x: usize, y: usize, rgb: [u8; 3]) {
        // x/8xw 0x100000
        unsafe {
            let dbg: &mut [u64] = core::slice::from_raw_parts_mut((0x100000) as *mut u64, 4);
            dbg[0] = write_pixel_rgb as u64;
            dbg[1] = write_pixel_bgr as u64;
            dbg[2] = self.writer as u64;
            dbg[3] = 0;
        }
        unsafe {
            (self.writer)(
                self.pixels_per_scan_line,
                self.horizontal_resolution,
                self.vertical_resolution,
                self.base,
                x,
                y,
                rgb,
            );
        }
    }
    pub fn get_horizontal_resolution(&self) -> usize {
        self.horizontal_resolution
    }
    pub fn get_vertical_resolution(&self) -> usize {
        self.vertical_resolution
    }
    pub unsafe fn heap_clone(&self, heap: *mut FrameBufferConfig) {
        (*heap).base = self.base;
        (*heap).pixels_per_scan_line = self.pixels_per_scan_line;
        (*heap).horizontal_resolution = self.horizontal_resolution;
        (*heap).vertical_resolution = self.vertical_resolution;
        (*heap).pixel_format = self.pixel_format;
        (*heap).writer = self.writer
    }
}
