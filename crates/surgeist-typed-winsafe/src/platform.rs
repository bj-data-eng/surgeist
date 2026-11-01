use super::{Error, Monitor, Point, Rect, Result, Window, collect_bounded};
use winsafe::{self as w, prelude::*};

fn error(operation: &'static str, code: w::co::ERROR) -> Error {
    Error::Native {
        operation,
        code: code.raw(),
    }
}

fn rect(value: w::RECT) -> Rect {
    Rect {
        left: value.left,
        top: value.top,
        right: value.right,
        bottom: value.bottom,
    }
}

fn monitor(native: &w::HMONITOR) -> Result<Monitor> {
    let info = native
        .GetMonitorInfo()
        .map_err(|code| error("GetMonitorInfoW", code))?;
    Ok(Monitor {
        device_name: info.szDevice(),
        frame: rect(info.rcMonitor),
        work_area: rect(info.rcWork),
    })
}

pub(super) fn monitors() -> Result<Vec<Monitor>> {
    let mut values = Vec::new();
    let mut failure = None;
    w::HDC::NULL
        .EnumDisplayMonitors(None, |native, _, _| {
            match monitor(&native) {
                Ok(value) => values.push(value),
                Err(error) => {
                    failure.get_or_insert(error);
                }
            }
            true
        })
        .map_err(|code| error("EnumDisplayMonitors", code))?;
    match failure {
        Some(error) => Err(error),
        None => Ok(values),
    }
}

pub(super) fn display_interfaces(device_name: &str) -> Result<Vec<String>> {
    let mut devices = w::EnumDisplayDevices(
        Some(device_name),
        Some(w::co::EDD::GET_DEVICE_INTERFACE_NAME),
    );
    collect_bounded(
        std::iter::from_fn(|| {
            w::SetLastError(w::co::ERROR::SUCCESS);
            devices.next().map(|device| {
                device
                    .map(|device| device.DeviceID())
                    .map_err(|code| error("EnumDisplayDevicesW", code))
            })
        }),
        128,
    )
}

pub(super) fn top_level_windows() -> Result<Vec<Window>> {
    let mut windows = Vec::new();
    w::EnumWindows(|native| {
        windows.push(Window { native });
        true
    })
    .map_err(|code| error("EnumWindows", code))?;
    Ok(windows)
}

impl Window {
    /// Reads the native window title; an empty title is a valid result.
    pub fn title(&self) -> Result<String> {
        w::SetLastError(w::co::ERROR::SUCCESS);
        self.native
            .GetWindowText()
            .map_err(|code| error("GetWindowTextW", code))
    }

    /// Reads the owning process identifier, rejecting a failed native lookup.
    pub fn process_id(&self) -> Result<u32> {
        w::SetLastError(w::co::ERROR::SUCCESS);
        let (thread, process) = self.native.GetWindowThreadProcessId();
        if thread == 0 || process == 0 {
            return Err(error("GetWindowThreadProcessId", w::GetLastError()));
        }
        Ok(process)
    }

    /// Reads the containing monitor using `MONITOR_DEFAULTTONULL`.
    pub fn monitor(&self) -> Result<Option<Monitor>> {
        let native = self.native.MonitorFromWindow(w::co::MONITOR::DEFAULTTONULL);
        if native == w::HMONITOR::NULL {
            return Ok(None);
        }
        monitor(&native).map(Some)
    }

    /// Reads client rectangle edges in native client coordinates.
    pub fn client_rect(&self) -> Result<Rect> {
        self.native
            .GetClientRect()
            .map(rect)
            .map_err(|code| error("GetClientRect", code))
    }

    /// Reads the client origin in screen coordinates under current DPI awareness.
    pub fn client_origin(&self) -> Result<Point> {
        self.native
            .ClientToScreen(w::POINT::default())
            .map(|point| Point {
                x: point.x,
                y: point.y,
            })
            .map_err(|code| error("ClientToScreen", code))
    }

    /// Reads the window DPI according to the window's DPI awareness.
    pub fn dpi(&self) -> Result<u32> {
        let dpi = self.native.GetDpiForWindow();
        if dpi == 0 {
            return Err(Error::InvalidData);
        }
        Ok(dpi)
    }
}
