use std::sync::Mutex;
use tauri::{LogicalSize, WebviewWindow};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Profile {
    Login,
    Setup,
    Workshop,
}

impl Profile {
    pub(crate) fn for_session(authenticated: bool, configured: bool) -> Self {
        if authenticated {
            Self::Workshop
        } else if configured {
            Self::Login
        } else {
            Self::Setup
        }
    }
    fn size(self) -> (f64, f64) {
        match self {
            Self::Login => (1000., 680.),
            Self::Setup => (1000., 760.),
            Self::Workshop => (1280., 760.),
        }
    }
    fn minimum(self) -> (f64, f64) {
        if self == Self::Workshop {
            (1120., 680.)
        } else {
            self.size()
        }
    }
}

trait WindowTarget {
    fn unmaximize(&self) -> Result<(), String>;
    fn minimum(&self, size: Option<(f64, f64)>) -> Result<(), String>;
    fn maximum(&self, size: Option<(f64, f64)>) -> Result<(), String>;
    fn resize(&self, size: (f64, f64)) -> Result<(), String>;
    fn resizable(&self, enabled: bool) -> Result<(), String>;
    fn maximizable(&self, enabled: bool) -> Result<(), String>;
    fn center(&self) -> Result<(), String>;
}

impl WindowTarget for WebviewWindow {
    fn unmaximize(&self) -> Result<(), String> {
        self.unmaximize().map_err(|e| e.to_string())
    }
    fn minimum(&self, size: Option<(f64, f64)>) -> Result<(), String> {
        self.set_min_size(size.map(|(w, h)| LogicalSize::new(w, h)))
            .map_err(|e| e.to_string())
    }
    fn maximum(&self, size: Option<(f64, f64)>) -> Result<(), String> {
        self.set_max_size(size.map(|(w, h)| LogicalSize::new(w, h)))
            .map_err(|e| e.to_string())
    }
    fn resize(&self, (w, h): (f64, f64)) -> Result<(), String> {
        self.set_size(LogicalSize::new(w, h))
            .map_err(|e| e.to_string())
    }
    fn resizable(&self, enabled: bool) -> Result<(), String> {
        self.set_resizable(enabled).map_err(|e| e.to_string())
    }
    fn maximizable(&self, enabled: bool) -> Result<(), String> {
        self.set_maximizable(enabled).map_err(|e| e.to_string())
    }
    fn center(&self) -> Result<(), String> {
        self.center().map_err(|e| e.to_string())
    }
}

#[derive(Default)]
pub struct WindowProfileState(Mutex<Option<Profile>>);

impl WindowProfileState {
    pub(crate) fn apply(&self, window: &WebviewWindow, profile: Profile) -> Result<(), String> {
        self.apply_to(window, profile)
            .map_err(|error| format!("No se pudo ajustar el tamaño de la ventana: {error}"))
    }
    fn apply_to(&self, window: &impl WindowTarget, profile: Profile) -> Result<(), String> {
        let mut previous = self
            .0
            .lock()
            .map_err(|_| "No se pudo consultar el tamaño de ventana.")?;
        if *previous == Some(profile) {
            return Ok(());
        }
        window.unmaximize()?;
        window.maximum(None)?;
        window.minimum(None)?;
        window.resizable(true)?;
        window.resize(profile.size())?;
        window.minimum(Some(profile.minimum()))?;
        window.maximum(if profile == Profile::Workshop {
            None
        } else {
            Some(profile.size())
        })?;
        window.maximizable(profile == Profile::Workshop)?;
        window.resizable(profile == Profile::Workshop)?;
        window.center()?;
        *previous = Some(profile);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    #[derive(Default)]
    struct Geometry {
        size: (f64, f64),
        min: Option<(f64, f64)>,
        max: Option<(f64, f64)>,
        resizable: bool,
        maximizable: bool,
        maximized: bool,
        centers: usize,
    }
    #[derive(Default)]
    struct Window(RefCell<Geometry>);
    impl WindowTarget for Window {
        fn unmaximize(&self) -> Result<(), String> {
            self.0.borrow_mut().maximized = false;
            Ok(())
        }
        fn minimum(&self, size: Option<(f64, f64)>) -> Result<(), String> {
            self.0.borrow_mut().min = size;
            Ok(())
        }
        fn maximum(&self, size: Option<(f64, f64)>) -> Result<(), String> {
            self.0.borrow_mut().max = size;
            Ok(())
        }
        fn resize(&self, size: (f64, f64)) -> Result<(), String> {
            let mut w = self.0.borrow_mut();
            let min = w.min.unwrap_or((0., 0.));
            let max = w.max.unwrap_or((f64::MAX, f64::MAX));
            w.size = (size.0.clamp(min.0, max.0), size.1.clamp(min.1, max.1));
            Ok(())
        }
        fn resizable(&self, v: bool) -> Result<(), String> {
            self.0.borrow_mut().resizable = v;
            Ok(())
        }
        fn maximizable(&self, v: bool) -> Result<(), String> {
            self.0.borrow_mut().maximizable = v;
            Ok(())
        }
        fn center(&self) -> Result<(), String> {
            self.0.borrow_mut().centers += 1;
            Ok(())
        }
    }
    #[test]
    fn login_size_returns_after_logout_even_from_a_maximized_workshop() {
        let state = WindowProfileState::default();
        let window = Window::default();
        state
            .apply_to(&window, Profile::for_session(false, true))
            .unwrap();
        assert_eq!(window.0.borrow().size, (1000., 680.));
        assert_eq!(window.0.borrow().min, window.0.borrow().max);
        assert!(!window.0.borrow().resizable);
        state
            .apply_to(&window, Profile::for_session(true, true))
            .unwrap();
        assert_eq!(window.0.borrow().size, (1280., 760.));
        assert_eq!(window.0.borrow().min, Some((1120., 680.)));
        assert!(window.0.borrow().max.is_none());
        window.0.borrow_mut().size = (1600., 900.);
        window.0.borrow_mut().maximized = true;
        state.apply_to(&window, Profile::Workshop).unwrap();
        assert_eq!(window.0.borrow().size, (1600., 900.));
        state
            .apply_to(&window, Profile::for_session(false, true))
            .unwrap();
        assert_eq!(window.0.borrow().size, (1000., 680.));
        assert!(!window.0.borrow().maximized);
        assert!(!window.0.borrow().maximizable);
        assert_eq!(window.0.borrow().centers, 3);
    }
    #[test]
    fn first_setup_keeps_its_size_between_its_two_steps() {
        let state = WindowProfileState::default();
        let window = Window::default();
        for _ in 0..2 {
            state
                .apply_to(&window, Profile::for_session(false, false))
                .unwrap();
        }
        assert_eq!(window.0.borrow().size, (1000., 760.));
        assert_eq!(window.0.borrow().centers, 1);
    }
}
