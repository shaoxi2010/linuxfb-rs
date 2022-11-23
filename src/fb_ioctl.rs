use nix::{libc, ioctl_read_bad, ioctl_write_ptr_bad};
use super::fb_defines::{FixScreeninfo, VarScreeninfo};


///IOCTL as defined in /usr/include/linux/fb.h
pub const FBIOGET_VSCREENINFO: libc::c_ulong = 0x4600;
pub const FBIOPUT_VSCREENINFO: libc::c_ulong = 0x4601;
pub const FBIOGET_FSCREENINFO: libc::c_ulong = 0x4602;
pub const FBIOPAN_DISPLAY: libc::c_ulong = 0x4606;

ioctl_read_bad!(fb_get_vinfo, FBIOGET_VSCREENINFO, VarScreeninfo);
ioctl_write_ptr_bad!(fb_set_vinfo, FBIOPUT_VSCREENINFO, VarScreeninfo);
ioctl_read_bad!(fb_get_finfo, FBIOGET_FSCREENINFO, FixScreeninfo);
ioctl_write_ptr_bad!(fb_pan_display, FBIOPAN_DISPLAY, VarScreeninfo);
