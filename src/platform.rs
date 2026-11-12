//! The few places where background mode needs the operating system directly.
//!
//! Only macOS needs anything: an app without a window should also leave the Dock,
//! and launching a running app again arrives as an Apple event instead of a second
//! process.

use eframe::UserEvent;
use winit::event_loop::EventLoopBuilder;

use crate::command::CommandSender;

#[cfg(target_os = "macos")]
mod macos {
    use objc2::rc::Retained;
    use objc2::runtime::{AnyObject, NSObject};
    use objc2::{AnyThread, DefinedClass, MainThreadMarker, define_class, msg_send, sel};
    use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
    use objc2_foundation::NSAppleEventManager;

    use crate::command::{Command, CommandSender};

    /// `kCoreEventClass` and `kAEReopenApplication`: the Apple event sent to a running
    /// app when the user opens it again (Finder, Spotlight, Dock, `open`).
    const CORE_EVENT_CLASS: u32 = u32::from_be_bytes(*b"aevt");
    const REOPEN_APPLICATION: u32 = u32::from_be_bytes(*b"rapp");

    define_class!(
        // SAFETY: NSObject has no subclassing requirements and the class has no Drop.
        #[unsafe(super(NSObject))]
        #[name = "ISMReopenHandler"]
        #[ivars = CommandSender]
        struct ReopenHandler;

        impl ReopenHandler {
            #[unsafe(method(handleReopen:withReplyEvent:))]
            fn handle_reopen(&self, _event: &AnyObject, _reply: &AnyObject) {
                self.ivars().send(Command::Open);
            }
        }
    );

    fn app() -> Option<Retained<NSApplication>> {
        MainThreadMarker::new().map(NSApplication::sharedApplication)
    }

    pub fn show_in_dock(visible: bool) {
        if let Some(app) = app() {
            app.setActivationPolicy(if visible {
                NSApplicationActivationPolicy::Regular
            } else {
                NSApplicationActivationPolicy::Accessory
            });
        }
    }

    pub fn bring_to_front() {
        if let Some(app) = app() {
            // `activate` replaces this from macOS 14; the app supports 10.15.
            #[allow(deprecated)]
            app.activateIgnoringOtherApps(true);
        }
    }

    pub fn on_reopen(commands: CommandSender) {
        let handler = ReopenHandler::alloc().set_ivars(commands);
        // SAFETY: `init` is the designated initialiser of the NSObject superclass.
        let handler: Retained<ReopenHandler> = unsafe { msg_send![super(handler), init] };
        let manager = NSAppleEventManager::sharedAppleEventManager();
        // SAFETY: the handler implements the selector with the two object arguments
        // the event manager passes, and the event codes are four-character codes.
        unsafe {
            let _: () = msg_send![
                &manager,
                setEventHandler: &*handler,
                andSelector: sel!(handleReopen:withReplyEvent:),
                forEventClass: CORE_EVENT_CLASS,
                andEventID: REOPEN_APPLICATION,
            ];
        }
        // The event manager does not retain its handlers; this one lives as long as
        // the process.
        std::mem::forget(handler);
    }
}

/// Apply start-up options to the event loop. A background start must not put the
/// app in the Dock or take focus.
pub fn configure_event_loop(builder: &mut EventLoopBuilder<UserEvent>, background: bool) {
    #[cfg(target_os = "macos")]
    {
        use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS as _};
        builder
            .with_activation_policy(if background {
                ActivationPolicy::Accessory
            } else {
                ActivationPolicy::Regular
            })
            .with_activate_ignoring_other_apps(!background);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (builder, background);
}

/// Show or hide the Dock icon (macOS). The app has one only while its window is open.
pub fn show_in_dock(visible: bool) {
    #[cfg(target_os = "macos")]
    macos::show_in_dock(visible);
    #[cfg(not(target_os = "macos"))]
    let _ = visible;
}

/// Make the app the active one so a window opened from the tray is in front (macOS).
pub fn bring_to_front() {
    #[cfg(target_os = "macos")]
    macos::bring_to_front();
}

/// Open the window when the user launches the already running app again (macOS;
/// elsewhere a second launch is a second process, see `instance.rs`).
///
/// Call on the main thread once the event loop runs: AppKit installs its own handler
/// while the app finishes launching and would replace an earlier one.
pub fn on_reopen(commands: CommandSender) {
    #[cfg(target_os = "macos")]
    macos::on_reopen(commands);
    #[cfg(not(target_os = "macos"))]
    let _ = commands;
}
