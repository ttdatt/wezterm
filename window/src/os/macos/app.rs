use crate::connection::{dispatch_application_event, ConnectionOps};
use crate::macos::menu::RepresentedItem;
use crate::macos::{nsstring, nsstring_to_str};
use crate::menu::{Menu, MenuItem};
use crate::{ApplicationEvent, ApplicationSpawnTarget, Connection};
use cocoa::appkit::{NSApp, NSApplicationTerminateReply, NSFilenamesPboardType, NSPasteboard, NSStringPboardType};
use cocoa::base::{id, nil};
use cocoa::foundation::{NSArray, NSInteger, NSURL};
use config::keyassignment::KeyAssignment;
use config::WindowCloseConfirmation;
use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use objc::declare::ClassDecl;
use objc::rc::StrongPtr;
use objc::runtime::{Class, Object, Sel, BOOL, NO, YES};
use objc::*;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const CLS_NAME: &str = "WezTermAppDelegate";
const UNIX_EXECUTABLE_UTI: &str = "public.unix-executable";
const LS_ROLES_ALL: u32 = 0xffff_ffff;

#[link(name = "CoreServices", kind = "framework")]
extern "C" {
    fn LSCopyDefaultRoleHandlerForContentType(
        in_content_type: CFStringRef,
        in_role: u32,
    ) -> CFStringRef;
    fn LSSetDefaultRoleHandlerForContentType(
        in_content_type: CFStringRef,
        in_role: u32,
        in_handler_bundle_id: CFStringRef,
    ) -> i32;
}

fn show_warning_alert(message: &str, info: &str) {
    unsafe {
        let alert: id = msg_send![class!(NSAlert), alloc];
        let alert: id = msg_send![alert, init];
        let () = msg_send![alert, setMessageText: *nsstring(message)];
        let () = msg_send![alert, setInformativeText: *nsstring(info)];
        let () = msg_send![alert, runModal];
    }
}

fn bundle_identifier() -> Option<String> {
    unsafe {
        let bundle: id = msg_send![class!(NSBundle), mainBundle];
        let bundle_id: id = msg_send![bundle, bundleIdentifier];
        if bundle_id.is_null() {
            None
        } else {
            Some(nsstring_to_str(bundle_id).to_string())
        }
    }
}

fn default_terminal_bundle_identifier() -> Option<String> {
    let uti = CFString::new(UNIX_EXECUTABLE_UTI);
    unsafe {
        let bundle_id =
            LSCopyDefaultRoleHandlerForContentType(uti.as_concrete_TypeRef(), LS_ROLES_ALL);
        if bundle_id.is_null() {
            None
        } else {
            Some(CFString::wrap_under_create_rule(bundle_id).to_string())
        }
    }
}

pub fn supports_default_terminal_menu_item() -> bool {
    bundle_identifier().is_some()
}

fn normalize_service_path(path: &Path) -> PathBuf {
    if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent().unwrap_or(path).to_path_buf()
    }
}

fn service_paths_from_pasteboard(pasteboard: id) -> Vec<PathBuf> {
    let mut dirs = BTreeSet::new();

    unsafe {
        let nsurl_class: id = class!(NSURL) as *const Class as id;
        let url_classes = NSArray::arrayWithObject(nil, nsurl_class);
        let urls = pasteboard.readObjectsForClasses_options(url_classes, nil);
        if !urls.is_null() {
            for idx in 0..urls.count() {
                let url = urls.objectAtIndex(idx);
                let url = if url.hasDirectoryPath() == YES {
                    url
                } else {
                    url.URLByDeletingLastPathComponent()
                };
                let path = url.path();
                if !path.is_null() {
                    dirs.insert(PathBuf::from(nsstring_to_str(path)));
                }
            }
            if !dirs.is_empty() {
                return dirs.into_iter().collect();
            }
        }

        let filenames = NSPasteboard::propertyListForType(pasteboard, NSFilenamesPboardType);
        if !filenames.is_null() {
            for idx in 0..filenames.count() {
                let path = PathBuf::from(nsstring_to_str(filenames.objectAtIndex(idx)));
                dirs.insert(normalize_service_path(&path));
            }
            return dirs.into_iter().collect();
        }

        let string = pasteboard.stringForType(NSStringPboardType);
        if !string.is_null() {
            for line in nsstring_to_str(string).lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                dirs.insert(normalize_service_path(Path::new(line)));
            }
        }
    }

    dirs.into_iter().collect()
}

fn set_service_error(error: *mut *mut Object, message: &str) {
    if error.is_null() {
        return;
    }

    let message = nsstring(message);
    unsafe {
        let message: *mut Object = *message;
        let message: *mut Object = msg_send![message, autorelease];
        *error = message;
    }
}

fn dispatch_service_open(pasteboard: id, target: ApplicationSpawnTarget, error: *mut *mut Object) {
    if pasteboard.is_null() {
        set_service_error(error, "WezTerm could not read the selected files from Finder.");
        return;
    }

    let dirs = service_paths_from_pasteboard(pasteboard);
    if dirs.is_empty() {
        set_service_error(error, "WezTerm could not resolve a directory from the selected items.");
        return;
    }

    for cwd in dirs {
        dispatch_application_event(ApplicationEvent::SpawnCommandInNewWindowOrTab {
            cwd,
            target,
        });
    }
}

fn has_open_window() -> bool {
    if let Some(conn) = Connection::get() {
        !conn.windows.borrow().is_empty()
    } else {
        unsafe {
            let windows: id = msg_send![NSApp(), windows];
            let count: NSInteger = msg_send![windows, count];
            count != 0
        }
    }
}

fn should_skip_initial_window_for_service(this: &Object) -> bool {
    let became_active: BOOL = unsafe { *this.get_ivar("application_has_become_active") };
    became_active == NO && !has_open_window()
}

extern "C" fn application_should_terminate(
    _self: &mut Object,
    _sel: Sel,
    _app: *mut Object,
) -> u64 {
    log::debug!("application termination requested");
    unsafe {
        match config::configuration().window_close_confirmation {
            WindowCloseConfirmation::NeverPrompt => terminate_now(),
            WindowCloseConfirmation::AlwaysPrompt => {
                let alert: id = msg_send![class!(NSAlert), alloc];
                let alert: id = msg_send![alert, init];
                let message_text = nsstring("Terminate WezTerm?");
                let info_text = nsstring("Detach and close all panes and terminate wezterm?");
                let cancel = nsstring("Cancel");
                let ok = nsstring("Ok");

                let () = msg_send![alert, setMessageText: message_text];
                let () = msg_send![alert, setInformativeText: info_text];
                let () = msg_send![alert, addButtonWithTitle: cancel];
                let () = msg_send![alert, addButtonWithTitle: ok];
                #[allow(non_upper_case_globals)]
                const NSModalResponseCancel: NSInteger = 1000;
                #[allow(non_upper_case_globals, dead_code)]
                const NSModalResponseOK: NSInteger = 1001;
                let result: NSInteger = msg_send![alert, runModal];
                log::info!("alert result is {result}");

                if result == NSModalResponseCancel {
                    NSApplicationTerminateReply::NSTerminateCancel as u64
                } else {
                    terminate_now()
                }
            }
        }
    }
}

fn terminate_now() -> u64 {
    if let Some(conn) = Connection::get() {
        conn.terminate_message_loop();
    }
    NSApplicationTerminateReply::NSTerminateNow as u64
}

extern "C" fn application_will_finish_launching(
    _self: &mut Object,
    _sel: Sel,
    _notif: *mut Object,
) {
    log::debug!("application_will_finish_launching");
}

extern "C" fn application_did_finish_launching(this: &mut Object, _sel: Sel, _notif: *mut Object) {
    log::debug!("application_did_finish_launching");
    unsafe {
        (*this).set_ivar("launched", YES);
    }
}

extern "C" fn application_did_become_active(
    this: &mut Object,
    _sel: Sel,
    _notif: *mut Object,
) {
    let became_active: BOOL = unsafe { *this.get_ivar("application_has_become_active") };
    if became_active == YES {
        return;
    }

    unsafe {
        (*this).set_ivar("application_has_become_active", YES);
    }

    let skip_initial_window: BOOL = unsafe { *this.get_ivar("skip_initial_window_on_activate") };
    if skip_initial_window == YES {
        unsafe {
            (*this).set_ivar("skip_initial_window_on_activate", NO);
        }
        return;
    }

    if !has_open_window() {
        dispatch_application_event(ApplicationEvent::PerformKeyAssignment(
            KeyAssignment::SpawnWindow,
        ));
    }
}

extern "C" fn application_open_untitled_file(
    _this: &mut Object,
    _sel: Sel,
    _app: *mut Object,
) -> BOOL {
    log::debug!("application_open_untitled_file");
    YES
}

extern "C" fn application_should_handle_reopen(
    this: &mut Object,
    _sel: Sel,
    _app: *mut Object,
    has_visible_windows: BOOL,
) -> BOOL {
    if has_visible_windows == YES || has_open_window() {
        return YES;
    }

    let became_active: BOOL = unsafe { *this.get_ivar("application_has_become_active") };
    if became_active == NO {
        return YES;
    }

    dispatch_application_event(ApplicationEvent::PerformKeyAssignment(
        KeyAssignment::SpawnWindow,
    ));

    NO
}

extern "C" fn wezterm_perform_key_assignment(
    _self: &mut Object,
    _sel: Sel,
    menu_item: *mut Object,
) {
    let menu_item = crate::os::macos::menu::MenuItem::with_menu_item(menu_item);
    // Safe because weztermPerformKeyAssignment: is only used with KeyAssignment
    let action = menu_item.get_represented_item();
    log::debug!("wezterm_perform_key_assignment {action:?}",);
    match action {
        Some(RepresentedItem::KeyAssignment(action)) => {
            dispatch_application_event(ApplicationEvent::PerformKeyAssignment(action));
        }
        None => {}
    }
}

extern "C" fn application_open_file(
    this: &mut Object,
    _sel: Sel,
    _app: *mut Object,
    file_name: *mut Object,
) {
    let launched: BOOL = unsafe { *this.get_ivar("launched") };
    if launched == YES {
        let file_name = unsafe { nsstring_to_str(file_name) }.to_string();
        log::debug!("application_open_file {file_name}");
        dispatch_application_event(ApplicationEvent::OpenCommandScript(file_name));
    }
}

extern "C" fn wezterm_open_tab_here(
    this: &mut Object,
    _sel: Sel,
    pasteboard: *mut Object,
    _user_data: *mut Object,
    error: *mut Object,
) {
    if should_skip_initial_window_for_service(this) {
        unsafe {
            (*this).set_ivar("skip_initial_window_on_activate", YES);
        }
    }
    dispatch_service_open(
        pasteboard,
        ApplicationSpawnTarget::Tab,
        error as *mut *mut Object,
    );
}

extern "C" fn wezterm_open_window_here(
    this: &mut Object,
    _sel: Sel,
    pasteboard: *mut Object,
    _user_data: *mut Object,
    error: *mut Object,
) {
    if should_skip_initial_window_for_service(this) {
        unsafe {
            (*this).set_ivar("skip_initial_window_on_activate", YES);
        }
    }
    dispatch_service_open(
        pasteboard,
        ApplicationSpawnTarget::Window,
        error as *mut *mut Object,
    );
}

extern "C" fn application_dock_menu(
    _self: &mut Object,
    _sel: Sel,
    _app: *mut Object,
) -> *mut Object {
    let dock_menu = Menu::new_with_title("");
    let new_window_item =
        MenuItem::new_with("New Window", Some(sel!(weztermPerformKeyAssignment:)), "");
    new_window_item
        .set_represented_item(RepresentedItem::KeyAssignment(KeyAssignment::SpawnWindow));
    dock_menu.add_item(&new_window_item);
    dock_menu.autorelease()
}

extern "C" fn wezterm_set_as_default_terminal(
    _self: &mut Object,
    _sel: Sel,
    _menu_item: *mut Object,
) {
    let Some(bundle_id) = bundle_identifier() else {
        show_warning_alert(
            "Failed to Set Default Terminal",
            "WezTerm could not determine its bundle identifier, so it could not be set as the default terminal application.",
        );
        return;
    };

    let uti = CFString::new(UNIX_EXECUTABLE_UTI);
    let bundle_id = CFString::new(&bundle_id);
    let status = unsafe {
        LSSetDefaultRoleHandlerForContentType(
            uti.as_concrete_TypeRef(),
            LS_ROLES_ALL,
            bundle_id.as_concrete_TypeRef(),
        )
    };

    if status != 0 {
        show_warning_alert(
            "Failed to Set Default Terminal",
            &format!(
                "WezTerm could not be set as the default terminal application.\n\nLaunchServices error: {status}"
            ),
        );
    }
}

extern "C" fn validate_menu_item(
    _self: &mut Object,
    _sel: Sel,
    menu_item: *mut Object,
) -> BOOL {
    let action: Sel = unsafe { msg_send![menu_item, action] };

    if action == sel!(weztermSetAsDefaultTerminal:) {
        let Some(bundle_id) = bundle_identifier() else {
            return NO;
        };

        if let Some(default_bundle_id) = default_terminal_bundle_identifier() {
            if default_bundle_id == bundle_id {
                return NO;
            }
        }
    }

    YES
}

fn get_class() -> &'static Class {
    Class::get(CLS_NAME).unwrap_or_else(|| {
        let mut cls = ClassDecl::new(CLS_NAME, class!(NSObject))
            .expect("Unable to register application delegate class");

        cls.add_ivar::<BOOL>("launched");
        cls.add_ivar::<BOOL>("application_has_become_active");
        cls.add_ivar::<BOOL>("skip_initial_window_on_activate");

        unsafe {
            cls.add_method(
                sel!(applicationShouldTerminate:),
                application_should_terminate as extern "C" fn(&mut Object, Sel, *mut Object) -> u64,
            );
            cls.add_method(
                sel!(applicationWillFinishLaunching:),
                application_will_finish_launching as extern "C" fn(&mut Object, Sel, *mut Object),
            );
            cls.add_method(
                sel!(applicationDidFinishLaunching:),
                application_did_finish_launching as extern "C" fn(&mut Object, Sel, *mut Object),
            );
            cls.add_method(
                sel!(applicationDidBecomeActive:),
                application_did_become_active as extern "C" fn(&mut Object, Sel, *mut Object),
            );
            cls.add_method(
                sel!(application:openFile:),
                application_open_file as extern "C" fn(&mut Object, Sel, *mut Object, *mut Object),
            );
            cls.add_method(
                sel!(openTab:userData:error:),
                wezterm_open_tab_here
                    as extern "C" fn(
                        &mut Object,
                        Sel,
                        *mut Object,
                        *mut Object,
                        *mut Object,
                    ),
            );
            cls.add_method(
                sel!(openWindow:userData:error:),
                wezterm_open_window_here
                    as extern "C" fn(
                        &mut Object,
                        Sel,
                        *mut Object,
                        *mut Object,
                        *mut Object,
                    ),
            );
            cls.add_method(
                sel!(applicationDockMenu:),
                application_dock_menu
                    as extern "C" fn(&mut Object, Sel, *mut Object) -> *mut Object,
            );
            cls.add_method(
                sel!(weztermPerformKeyAssignment:),
                wezterm_perform_key_assignment as extern "C" fn(&mut Object, Sel, *mut Object),
            );
            cls.add_method(
                sel!(applicationOpenUntitledFile:),
                application_open_untitled_file
                    as extern "C" fn(&mut Object, Sel, *mut Object) -> BOOL,
            );
            cls.add_method(
                sel!(applicationShouldHandleReopen:hasVisibleWindows:),
                application_should_handle_reopen
                    as extern "C" fn(&mut Object, Sel, *mut Object, BOOL) -> BOOL,
            );
            cls.add_method(
                sel!(weztermSetAsDefaultTerminal:),
                wezterm_set_as_default_terminal as extern "C" fn(&mut Object, Sel, *mut Object),
            );
            cls.add_method(
                sel!(validateMenuItem:),
                validate_menu_item as extern "C" fn(&mut Object, Sel, *mut Object) -> BOOL,
            );
        }

        cls.register()
    })
}

pub fn create_app_delegate() -> StrongPtr {
    let cls = get_class();
    unsafe {
        let delegate: *mut Object = msg_send![cls, alloc];
        let delegate: *mut Object = msg_send![delegate, init];
        (*delegate).set_ivar("launched", NO);
        (*delegate).set_ivar("application_has_become_active", NO);
        (*delegate).set_ivar("skip_initial_window_on_activate", NO);
        StrongPtr::new(delegate)
    }
}
