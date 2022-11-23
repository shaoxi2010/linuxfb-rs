mod fb_defines;
mod fb_ioctl;
use fb_defines::*;
use fb_ioctl::*;

use std::{io, fs::{File, OpenOptions}, cell::RefCell, path::Path, slice::from_raw_parts_mut};
use memmap2::{MmapMut, MmapOptions};
use thiserror::Error;
use std::os::unix::io::AsRawFd;

#[derive(Error, Debug)]
pub enum FbError {
    #[error("fb device io error")]
    IoError(#[from] io::Error),
    #[error("Cant Get FB Vinfo")]
    GetVinfo,
    #[error("Cant Set FB Vinfo")]
    SetVinfo,
    #[error("Cant Get FB Finfo")]
    GetFinfo,
    #[error("Cant Set FB Finfo")]
    SetFinfo,
    #[error("Cant Set FB PANDISPLAY")]
    PanDisplay,
}

pub struct FrameBuffer {
    device:File,
    frame: RefCell<MmapMut>,
    vinfo: RefCell<VarScreeninfo>,
    finfo: FixScreeninfo,
    can_double: bool,
}

impl FrameBuffer {
    fn new(fb: &Path) -> Result<FrameBuffer, FbError> {
        let device = OpenOptions::new()
            .read(true)
            .write(true)
            .open(fb)?;

        let mut vinfo = VarScreeninfo::default();
        let mut finfo = FixScreeninfo::default();

        unsafe {
            if fb_get_finfo(device.as_raw_fd(), &mut finfo as *mut _).is_err() {
                return Err(FbError::GetFinfo);
            }
            if fb_get_vinfo(device.as_raw_fd(), &mut vinfo as *mut _).is_err() {
                return Err(FbError::GetVinfo);
            }
        }

        let can_double = finfo.smem_len >= finfo.line_length * vinfo.yres * 2;
        if can_double {
            vinfo.yres_virtual = vinfo.yres * 2;
            vinfo.yoffset = 0;
            unsafe {
                if fb_set_vinfo(device.as_raw_fd(), &vinfo as *const _).is_err() {
                    return Err(FbError::SetVinfo)
                }
            }
        }
        let frame = unsafe {MmapOptions::new().len(finfo.smem_len as usize).map_mut(&device)?};
        Ok(FrameBuffer{
            device, frame: RefCell::new(frame), vinfo: RefCell::new(vinfo), finfo, can_double
        })
    }

    fn screen_bytes(&self) -> usize {
        let vinfo = self.vinfo.borrow();
        (self.finfo.line_length * vinfo.yres) as usize
    }

    fn get_fb0_buffer(&self) -> Option<&mut [u8]> {
        let mut frame = self.frame.borrow_mut();
        let fb = unsafe {
            from_raw_parts_mut(frame.as_mut_ptr(), self.screen_bytes())
        };
        Some(fb)
    }

    fn get_fb1_buffer(&self) -> Option<&mut [u8]> {
        if self.can_double {
            let mut frame = self.frame.borrow_mut();
            let fb = unsafe {
                from_raw_parts_mut(frame.as_mut_ptr().offset(self.screen_bytes() as isize), self.screen_bytes())
            };
            Some(fb)
        } else {
            None
        }
    }

    pub fn color_depth(&self) -> usize {
        let vinfo = self.vinfo.borrow();
        vinfo.bits_per_pixel as usize
    }

    pub fn screen_size(&self) -> (usize, usize) {
        let vinfo = self.vinfo.borrow();
        (vinfo.xres as usize, vinfo.yres as usize)
    }

    pub fn swap(&self) -> Result<(), FbError> {
        if !self.can_double {
            return Ok(());
        }
        let mut vinfo = self.vinfo.borrow_mut();
        vinfo.yoffset = if vinfo.yoffset == 0 { vinfo.yres } else { 0 };

        unsafe {
            if fb_pan_display(self.device.as_raw_fd(), self.vinfo.as_ptr() as *const _).is_err() {
                return Err(FbError::PanDisplay);
            }
        }
        Ok(())
    }
}