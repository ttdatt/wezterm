use crate::macos::{file_paths_from_pasteboard, nsstring, nsstring_to_str};
use cocoa::appkit::{NSPasteboard, NSStringPboardType};
use cocoa::base::*;
use cocoa::foundation::NSArray;

pub struct Clipboard {
    pasteboard: id,
}

impl Clipboard {
    pub fn new() -> Self {
        let pasteboard = unsafe { NSPasteboard::generalPasteboard(nil) };
        if pasteboard.is_null() {
            panic!("NSPasteboard::generalPasteboard returned null");
        }
        Clipboard { pasteboard }
    }

    pub fn read(&self) -> anyhow::Result<String> {
        let paths = file_paths_from_pasteboard(self.pasteboard);
        if !paths.is_empty() {
            let filenames = paths
                .iter()
                .map(|path| {
                    let path = path.to_string_lossy().into_owned();
                    shlex::try_quote(&path)
                        .map(|quoted| quoted.into_owned())
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>();
            return Ok(filenames.join(" "));
        }

        unsafe {
            let s = self.pasteboard.stringForType(NSStringPboardType);
            if !s.is_null() {
                let str = nsstring_to_str(s);
                return Ok(str.to_string());
            }
        }
        anyhow::bail!("pasteboard read returned empty");
    }

    pub fn write(&mut self, data: String) -> anyhow::Result<()> {
        unsafe {
            self.pasteboard.clearContents();
            let success: BOOL = self
                .pasteboard
                .writeObjects(NSArray::arrayWithObject(nil, *nsstring(&data)));
            anyhow::ensure!(success == YES, "pasteboard write returned false");
            Ok(())
        }
    }
}
