//! Runs the app for the life of the process: with a window while the user has it
//! open, and with only the tray icon the rest of the time.
//!
//! The window is destroyed when it closes and built again when it is asked for. The
//! [`App`] and its monitor live outside it and keep running throughout. One winit
//! event loop serves both states, because the tray icon needs it in the background.
//!
//! Each state is one run of that loop, and winit closes all of the app's windows when
//! a run ends. That is what removes the main window, but on macOS it also disables
//! the tray icon, so every run after the first starts by reviving it.

use std::cell::Cell;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use eframe::UserEvent;
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::platform::run_on_demand::EventLoopExtRunOnDemand as _;
use winit::window::WindowId;

use crate::app::App;
use crate::command::{self, Command, CommandSender};
use crate::data_store::DataStore;
use crate::instance::{self, Listener};
use crate::paths::AppPaths;
use crate::tray::{self, Tray};
use crate::wake::Waker;
use crate::{APP_NAME, platform};

const DEFAULT_SIZE: egui::Vec2 = egui::vec2(1200.0, 800.0);
const MIN_SIZE: egui::Vec2 = egui::vec2(640.0, 480.0);

pub struct Launch {
    pub handle: tokio::runtime::Handle,
    pub paths: AppPaths,
    pub store: Arc<Mutex<DataStore>>,
    /// Answers later launches of the app; `None` when that could not be set up.
    pub listener: Option<Listener>,
    /// Start with the tray icon only.
    pub background: bool,
}

/// What exists once the event loop runs, for the rest of the process.
struct Fixtures {
    tray: Tray,
    commands: CommandSender,
    ready: bool,
}

impl Fixtures {
    /// Call once per event loop run, from inside the running loop, on the main thread.
    fn loop_started(&mut self) {
        if !self.ready {
            self.ready = true;
            self.tray.ensure();
            platform::on_reopen(self.commands.clone());
        } else {
            // The run that just ended took the tray icon's window with it.
            self.tray.revive();
        }
    }
}

pub fn run(launch: Launch) -> eframe::Result {
    let mut builder = EventLoop::<UserEvent>::with_user_event();
    platform::configure_event_loop(&mut builder, launch.background);
    let mut event_loop = builder.build()?;

    // Without a window nothing repaints, so background work wakes the event loop.
    let proxy = Mutex::new(event_loop.create_proxy());
    let waker = Waker::new(move || {
        let proxy = proxy.lock().unwrap_or_else(|p| p.into_inner());
        let _ = proxy.send_event(UserEvent::RequestRepaint {
            viewport_id: egui::ViewportId::ROOT,
            when: Instant::now(),
            cumulative_pass_nr: 0,
        });
    });

    let (sender, commands) = command::channel(waker.clone());
    tray::listen(sender.clone());
    if let Some(listener) = launch.listener {
        listener.serve(sender.clone());
    }

    let data_dir = launch.paths.data_dir.clone();
    let mut app = App::new(launch.handle, launch.paths, launch.store, waker);
    let mut fixtures = Fixtures {
        tray: Tray::default(),
        commands: sender,
        ready: false,
    };
    let window_size = Cell::new(DEFAULT_SIZE);
    let mut show_window = !launch.background;
    let mut first_window = true;
    if launch.background {
        log::info!("Started in the background");
    }

    let result = loop {
        if show_window {
            let shown = run_window(
                &mut event_loop,
                &mut app,
                &commands,
                &mut fixtures,
                &window_size,
            );
            match shown {
                // Nothing to fall back to when the very first window fails.
                Err(error) if first_window => break Err(error),
                Err(error) => log::error!("Could not open the window: {error}"),
                Ok(()) => {}
            }
            first_window = false;
            if app.quit_requested() {
                break Ok(());
            }
            if !fixtures.tray.is_available() {
                log::info!("No tray icon to return from: closing the window quits");
                break Ok(());
            }
            log::info!("Window closed, running in the background");
        }
        match run_background(&mut event_loop, &mut app, &commands, &mut fixtures) {
            Ok(Wanted::Window) => show_window = true,
            Ok(Wanted::Quit) => break Ok(()),
            Err(error) => break Err(error),
        }
    };

    app.shutdown();
    instance::release(&data_dir);
    result
}

// ----- Window -----

/// The eframe side of the app while a window is open.
struct Window<'a> {
    app: &'a mut App,
    fixtures: &'a mut Fixtures,
    commands: &'a Receiver<Command>,
    size: &'a Cell<egui::Vec2>,
    drew: &'a Cell<bool>,
}

impl eframe::App for Window<'_> {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.drew.set(true);
        let ctx = ui.ctx().clone();

        for command in self.commands.try_iter() {
            match command {
                Command::Open => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    platform::bring_to_front();
                }
                Command::Quit => self.app.request_quit(),
            }
        }

        self.app.frame(ui);
        self.fixtures.tray.update(self.app.tray_status());

        if self.app.quit_requested() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        self.remember_size(&ctx);
    }
}

impl Window<'_> {
    /// The next window opens at the size this one had.
    fn remember_size(&self, ctx: &egui::Context) {
        let size = ctx.input(|input| {
            let viewport = input.viewport();
            let special =
                [viewport.minimized, viewport.maximized, viewport.fullscreen].contains(&Some(true));
            viewport
                .inner_rect
                .filter(|_| !special)
                .map(|rect| rect.size())
        });
        if let Some(size) = size
            && size.x >= MIN_SIZE.x
            && size.y >= MIN_SIZE.y
        {
            self.size.set(size);
        }
    }
}

/// Open the window and return when it has closed.
fn run_window(
    event_loop: &mut EventLoop<UserEvent>,
    app: &mut App,
    commands: &Receiver<Command>,
    fixtures: &mut Fixtures,
    size: &Cell<egui::Vec2>,
) -> eframe::Result {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../docs/assets/icon-256x256.png"))
        .expect("bundled icon must be a valid PNG");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(APP_NAME)
            .with_app_id("com.omrilotan.internet-speed-monitor")
            .with_inner_size(size.get())
            .with_min_inner_size(MIN_SIZE)
            .with_icon(icon),
        ..Default::default()
    };

    let drew = Cell::new(false);
    let outcome = {
        let drew = &drew;
        let app = &mut *app;
        let mut window = eframe::create_native(
            APP_NAME,
            options,
            Box::new(move |cc| {
                fixtures.loop_started();
                platform::show_in_dock(true);
                platform::bring_to_front();
                app.attach(&cc.egui_ctx);
                Ok(Box::new(Window {
                    app,
                    fixtures,
                    commands,
                    size,
                    drew,
                }))
            }),
            event_loop,
        );
        event_loop.run_app_on_demand(&mut window)
    };
    app.detach();
    outcome?;

    // eframe logs why a window failed but does not hand the error over.
    if drew.get() {
        Ok(())
    } else {
        Err(eframe::Error::AppCreation(
            "the window could not be created".into(),
        ))
    }
}

// ----- Background -----

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Wanted {
    Window,
    Quit,
}

/// The event loop's handler while no window exists.
struct Background<'a> {
    app: &'a mut App,
    commands: &'a Receiver<Command>,
    fixtures: &'a mut Fixtures,
    started: bool,
    wanted: Option<Wanted>,
}

impl Background<'_> {
    fn poll(&mut self, event_loop: &ActiveEventLoop) {
        self.app.pump();
        self.fixtures.tray.update(self.app.tray_status());
        let mut wanted = None;
        for command in self.commands.try_iter() {
            match command {
                Command::Quit => wanted = Some(Wanted::Quit),
                Command::Open => wanted = wanted.or(Some(Wanted::Window)),
            }
        }
        if wanted.is_some() {
            self.wanted = wanted;
            event_loop.exit();
        }
    }
}

impl ApplicationHandler<UserEvent> for Background<'_> {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, _cause: StartCause) {
        // The window's loop may have left a timed or polling wake-up behind.
        event_loop.set_control_flow(ControlFlow::Wait);
        if !self.started {
            self.started = true;
            self.fixtures.loop_started();
            platform::show_in_dock(false);
        }
        self.poll(event_loop);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, _event: UserEvent) {
        self.poll(event_loop);
    }

    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, _: WindowEvent) {}
}

/// Run with only the tray icon until the window is asked for or the app quits.
fn run_background(
    event_loop: &mut EventLoop<UserEvent>,
    app: &mut App,
    commands: &Receiver<Command>,
    fixtures: &mut Fixtures,
) -> eframe::Result<Wanted> {
    let mut background = Background {
        app,
        commands,
        fixtures,
        started: false,
        wanted: None,
    };
    event_loop.run_app_on_demand(&mut background)?;
    // A loop that ends without a request was ended by the system: quit.
    Ok(background.wanted.unwrap_or(Wanted::Quit))
}
