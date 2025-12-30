use std::io;
use std::mem;
use std::os::unix::io::AsRawFd;

#[repr(C)]
struct TermSize {
    row: u16,
    col: u16,
    x_pixel: u16,
    y_pixel: u16,
}

unsafe extern "C" {
    fn ioctl(fd: i32, request: u64, ...) -> i32;
}

#[cfg(unix)]
pub fn get_terminal_rows() -> io::Result<usize> {
    const TIOCGWINSZ: u64 = 0x5413;

    let mut size: TermSize = unsafe { mem::zeroed() };
    let result = unsafe {
        ioctl(io::stdout().as_raw_fd(), TIOCGWINSZ, &mut size)
    };

    if result == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(size.row as usize)
    }
}
