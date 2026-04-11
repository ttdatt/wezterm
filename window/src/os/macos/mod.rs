#![allow(unexpected_cfgs)] // <https://github.com/SSheldon/rust-objc/issues/125>
use cocoa::base::{id, nil};
use cocoa::foundation::{NSArray, NSString};
use objc::rc::StrongPtr;
use objc::runtime::{Class, Object, BOOL, NO, YES};
use objc::*;
use std::path::PathBuf;

mod app;
pub mod bitmap;
pub mod clipboard;
pub mod connection;
pub mod menu;
pub mod window;

mod keycodes;

pub use self::window::*;
pub use bitmap::*;
pub use connection::*;
pub fn supports_default_terminal_menu_item() -> bool {
    app::supports_default_terminal_menu_item()
}

pub(crate) const NSPASTEBOARD_TYPE_FILE_URL: &str = "public.file-url";

/// Convert a rust string to a cocoa string
fn nsstring(s: &str) -> StrongPtr {
    unsafe { StrongPtr::new(NSString::alloc(nil).init_str(s)) }
}

unsafe fn nsstring_to_str<'a>(mut ns: *mut Object) -> &'a str {
    let is_astring: bool = msg_send![ns, isKindOfClass: class!(NSAttributedString)];
    if is_astring {
        ns = msg_send![ns, string];
    }
    let data = NSString::UTF8String(ns as id) as *const u8;
    let len = NSString::len(ns as id);
    let bytes = std::slice::from_raw_parts(data, len);
    std::str::from_utf8_unchecked(bytes)
}

/// Helper function to easily convert a Rust' bool to an objc' BOOL
fn to_yes_no(value: bool) -> BOOL {
    if value {
        YES
    } else {
        NO
    }
}

/// Helper function to easily convert an objc' BOOL to a Rust' bool
fn from_yes_no(value: BOOL) -> bool {
    value == YES
}

fn file_paths_from_pasteboard(pasteboard: id) -> Vec<PathBuf> {
    if pasteboard.is_null() {
        return vec![];
    }

    unsafe {
        let nsurl_class: id = class!(NSURL) as *const Class as id;
        let url_classes = NSArray::arrayWithObject(nil, nsurl_class);
        let urls: id = msg_send![pasteboard, readObjectsForClasses: url_classes options: nil];

        if urls.is_null() {
            return vec![];
        }

        let mut paths = Vec::new();
        for idx in 0..urls.count() {
            let url: id = urls.objectAtIndex(idx);
            let is_file_url: BOOL = msg_send![url, isFileURL];
            if is_file_url != YES {
                continue;
            }

            let path: id = msg_send![url, path];
            if !path.is_null() {
                paths.push(PathBuf::from(nsstring_to_str(path)));
            }
        }

        paths
    }
}
