use crate::{
    ClientWindowMetrics, HostEnvironment, OutputDevice, OutputResolution, PhysicalPoint,
    PhysicalSize, Point, Rect, Result, ScaleFactor, ScreenAreaSource, ScreenMetrics, Size,
    UserPreferences, ViewportMetrics, ViewportSizes,
};

/// Determines whether winit observations describe native content or a browser canvas.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CaptureProfile {
    NativeContent,
    BrowserCanvas,
}

/// Raw events retain the existing base capture; output enrichment needs a safe
/// committed snapshot at creation or idle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CaptureScope {
    Basic,
    Enriched,
}

/// Captures the native content host: logical units are CSS pixels at page zoom
/// one; there is no retractable UA chrome in this adapter. This does not infer
/// primary input, blending or browsing presentation. Monitor work area and
/// WindowServer color estimates are enriched only at creation and safe idle.
/// The caller supplies the complete native baseline, including the independent
/// system scheme. Event metrics supply scale/size even if getters lag.
pub(crate) fn observe_environment(
    window: &winit::window::Window,
    metrics: &crate::Metrics,
    preferences: UserPreferences,
    profile: CaptureProfile,
    scope: CaptureScope,
) -> Result<HostEnvironment> {
    let monitor = window.current_monitor();
    let native = NativeObservation {
        scale: metrics.scale_factor(),
        content: metrics.physical_size(),
        client_origin: window
            .inner_position()
            .ok()
            .map(|p| PhysicalPoint { x: p.x, y: p.y }),
        screen: monitor.as_ref().map(|monitor| {
            let origin = monitor.position();
            let size = monitor.size();
            (
                PhysicalPoint {
                    x: origin.x,
                    y: origin.y,
                },
                PhysicalSize {
                    width: size.width,
                    height: size.height,
                },
            )
        }),
        preferences,
    };
    let mut environment = native.into_environment(profile)?;
    if scope == CaptureScope::Enriched
        && profile == CaptureProfile::NativeContent
        && let (Some(monitor), Some(screen)) = (monitor, environment.screen.as_ref())
    {
        let facts = crate::native_monitor::observe(
            &monitor,
            screen.area().origin,
            ScaleFactor::new(metrics.scale_factor())?,
        );
        environment.screen = Some(ScreenMetrics::new(
            screen.area(),
            facts.available_area,
            screen.source(),
            facts.color_depth,
            facts.color_component_bits,
        )?);
    }
    Ok(environment)
}

/// This native observer does not produce these independent host facts.
pub(crate) fn preserve_independent(
    mut observed: HostEnvironment,
    previous: &HostEnvironment,
) -> HostEnvironment {
    observed.input = previous.input.clone();
    observed.navigation = previous.navigation;
    observed.display_mode = previous.display_mode;
    observed
}

struct NativeObservation {
    scale: f64,
    content: PhysicalSize,
    client_origin: Option<PhysicalPoint>,
    screen: Option<(PhysicalPoint, PhysicalSize)>,
    preferences: UserPreferences,
}

impl NativeObservation {
    fn into_environment(self, profile: CaptureProfile) -> Result<HostEnvironment> {
        if profile == CaptureProfile::BrowserCanvas {
            // Canvas size/position and devicePixelRatio cannot establish browser
            // viewport, desktop coordinates, native scale or separate page zoom.
            return Ok(HostEnvironment {
                preferences: self.preferences,
                ..HostEnvironment::default()
            });
        }
        let scale = ScaleFactor::new(self.scale)?;
        let size = |size: PhysicalSize| Size {
            width: f64::from(size.width) / scale.get(),
            height: f64::from(size.height) / scale.get(),
        };
        let point = |point: PhysicalPoint| Point {
            x: f64::from(point.x) / scale.get(),
            y: f64::from(point.y) / scale.get(),
        };
        let content = size(self.content);
        let screen = self
            .screen
            .map(|(origin, extent)| {
                ScreenMetrics::new(
                    Rect {
                        origin: point(origin),
                        size: size(extent),
                    },
                    None,
                    ScreenAreaSource::OutputDevice,
                    None,
                    None,
                )
            })
            .transpose()?;
        let client_area = self
            .client_origin
            .map(|origin| {
                ClientWindowMetrics::new(Rect {
                    origin: point(origin),
                    size: content,
                })
            })
            .transpose()?;
        Ok(HostEnvironment {
            native_scale: Some(scale),
            page_zoom: Some(ScaleFactor::ONE),
            output: if screen.is_some() {
                OutputDevice::Present(OutputResolution::Raster(scale))
            } else {
                OutputDevice::Unavailable
            },
            screen,
            client_area,
            viewport: Some(ViewportMetrics::new(
                content,
                ViewportSizes::fixed(content)?,
            )?),
            preferences: self.preferences,
            ..HostEnvironment::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Theme;

    #[test]
    fn native_capture_converts_screen_and_client_origin_in_one_coordinate_space() {
        let environment = NativeObservation {
            scale: 2.0,
            content: PhysicalSize {
                width: 1600,
                height: 1200,
            },
            client_origin: Some(PhysicalPoint { x: -3600, y: 80 }),
            screen: Some((
                PhysicalPoint { x: -3840, y: 0 },
                PhysicalSize {
                    width: 3840,
                    height: 2160,
                },
            )),
            preferences: UserPreferences {
                color_scheme: Some(Theme::Dark),
                ..UserPreferences::default()
            },
        }
        .into_environment(CaptureProfile::NativeContent)
        .unwrap();
        let screen = environment.screen.as_ref().unwrap();
        assert_eq!(screen.area().origin, Point { x: -1920.0, y: 0.0 });
        assert_eq!(
            screen.area().size,
            Size {
                width: 1920.0,
                height: 1080.0
            }
        );
        assert_eq!(screen.available_area(), None);
        assert_eq!(
            environment.client_area.unwrap().area().origin,
            Point {
                x: -1800.0,
                y: 40.0
            }
        );
        let viewport = environment.viewport.unwrap();
        assert_eq!(
            viewport.layout(),
            Size {
                width: 800.0,
                height: 600.0
            }
        );
        assert_eq!(viewport.sizes().large(), viewport.sizes().small());
        assert_eq!(viewport.sizes().dynamic(), viewport.layout());
        assert_eq!(environment.preferences.color_scheme, Some(Theme::Dark));
        assert_eq!(environment.preferences.reduced_motion, None);
        assert_eq!(environment.blending, None);
        assert_eq!(environment.input, None);
    }

    #[test]
    fn browser_canvas_observations_do_not_claim_desktop_or_viewport_facts() {
        let environment = NativeObservation {
            scale: 3.0,
            content: PhysicalSize {
                width: 1200,
                height: 900,
            },
            client_origin: Some(PhysicalPoint { x: 40, y: 80 }),
            screen: Some((
                PhysicalPoint { x: 0, y: 0 },
                PhysicalSize {
                    width: 3840,
                    height: 2160,
                },
            )),
            preferences: UserPreferences {
                color_scheme: Some(Theme::Dark),
                ..UserPreferences::default()
            },
        }
        .into_environment(CaptureProfile::BrowserCanvas)
        .unwrap();
        assert_eq!(environment.native_scale, None);
        assert_eq!(environment.page_zoom, None);
        assert_eq!(environment.viewport, None);
        assert_eq!(environment.screen, None);
        assert_eq!(environment.client_area, None);
        assert_eq!(environment.output, OutputDevice::Unavailable);
        assert_eq!(environment.preferences.color_scheme, Some(Theme::Dark));
    }

    #[test]
    fn geometry_capture_keeps_the_complete_independent_baseline() {
        let preferences = UserPreferences {
            color_scheme: Some(Theme::Dark),
            reduced_motion: Some(crate::ReductionPreference::Reduce),
            reduced_transparency: Some(crate::ReductionPreference::NoPreference),
            contrast: Some(crate::ContrastPreference::More),
            reduced_data: None,
        };
        for profile in [CaptureProfile::NativeContent, CaptureProfile::BrowserCanvas] {
            let environment = NativeObservation {
                scale: 2.0,
                content: PhysicalSize {
                    width: 800,
                    height: 600,
                },
                client_origin: None,
                screen: None,
                preferences: preferences.clone(),
            }
            .into_environment(profile)
            .unwrap();
            assert_eq!(environment.preferences, preferences);
        }
    }

    #[test]
    fn unavailable_native_observations_do_not_become_os_facts() {
        let environment = NativeObservation {
            scale: 1.0,
            content: PhysicalSize {
                width: 0,
                height: 0,
            },
            client_origin: None,
            screen: None,
            preferences: UserPreferences::default(),
        }
        .into_environment(CaptureProfile::NativeContent)
        .unwrap();
        assert_eq!(environment.screen, None);
        assert_eq!(environment.client_area, None);
        assert_eq!(environment.output, OutputDevice::Unavailable);
        assert_eq!(environment.preferences.color_scheme, None);
        assert_eq!(environment.viewport.unwrap().layout(), Size::default());
    }
}
