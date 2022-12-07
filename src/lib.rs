mod fb_defines;
mod fb_ioctl;
use fb_defines::*;
use fb_ioctl::*;

use std::{io, fs::{File, OpenOptions}, path::Path};
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

enum Frames {
    Double(MmapMut, MmapMut),
    Singel(MmapMut),
}

pub struct FrameBuffer {
    device:File,
    frames: Frames,
    vinfo: VarScreeninfo,
}

impl FrameBuffer {
    pub fn new(fb: &Path) -> Result<FrameBuffer, FbError> {
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
        let screen_size = finfo.line_length * vinfo.yres;
        if finfo.smem_len >= screen_size * 2 {
            
            vinfo.yoffset = 0;
            unsafe {
                if vinfo.yres_virtual < vinfo.yres * 2 {
                    vinfo.yres_virtual = vinfo.yres * 2;
                    if fb_set_vinfo(device.as_raw_fd(), &vinfo).is_err() {
                        return Err(FbError::SetVinfo)
                    }
                } else {
                    if fb_pan_display(device.as_raw_fd(), &vinfo).is_err() {
                        return Err(FbError::PanDisplay);
                    }
                }

            }
            let frames = Frames::Double(
                unsafe {
                    MmapOptions::new().len(screen_size as usize).map_mut(&device)?
                },
                unsafe {
                    MmapOptions::new().offset(screen_size as u64).len(screen_size as usize).map_mut(&device)?
                }
            );
            Ok(Self{ device, frames, vinfo })
        } else {
            let frames = Frames::Singel(unsafe {
                MmapOptions::new().len(screen_size as usize).map_mut(&device)?
            });
            Ok(Self{ device, frames, vinfo })
        }
    }

    pub fn get_disp_data(&mut self) -> &mut [u8] {
        match &mut self.frames {
            Frames::Double(disp, _) => disp.as_mut(),
            Frames::Singel(disp) => disp.as_mut(),
        }
    }

    pub fn get_buff_data(&mut self) -> Option<&mut [u8]> {
        match &mut self.frames {
            Frames::Double(_, buff) => Some(buff.as_mut()),
            Frames::Singel(_) => None,
        }
    }

    pub fn color_depth(&self) -> usize {
        self.vinfo.bits_per_pixel as usize
    }

    pub fn screen_size(&self) -> (usize, usize) {
        (self.vinfo.xres as usize, self.vinfo.yres as usize)
    }

    pub fn swap(& mut self) -> Result<(), FbError> {
        if let Frames::Double(disp, buff) = &mut self.frames {
            self.vinfo.yoffset = if self.vinfo.yoffset == 0 { self.vinfo.yres } else { 0 };
            unsafe {
                if fb_pan_display(self.device.as_raw_fd(), &self.vinfo).is_err() {
                    return Err(FbError::PanDisplay);
                }
            }
            std::mem::swap( disp,  buff);
        }

        Ok(())
    }
}