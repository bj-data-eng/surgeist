use surgeist::render::{
    Attachment, Color, Layer, Options, Parameters, Point, Radii, Rect, Renderer, Scene, Shadow,
    Size, Stroke, Surface, SurfaceOptions,
};
use surgeist::window;

struct RenderWindow {
    renderer: Renderer,
    surface: Option<Surface>,
}

impl RenderWindow {
    fn new(renderer: Renderer) -> Self {
        Self {
            renderer,
            surface: None,
        }
    }

    fn attach_surface(&mut self, handle: window::Handle, metrics: &window::Metrics) {
        let options = SurfaceOptions {
            size: Size::new(metrics.logical_size.width, metrics.logical_size.height),
            scale: metrics.scale_factor,
            ..SurfaceOptions::default()
        };
        self.surface = Some(
            self.renderer
                .create_surface(Attachment::from_window(handle), options)
                .expect("render surface should attach to native window"),
        );
    }

    fn resize_surface(&mut self, metrics: &window::Metrics) {
        if let Some(surface) = &mut self.surface {
            surface
                .resize(
                    Size::new(metrics.logical_size.width, metrics.logical_size.height),
                    metrics.scale_factor,
                )
                .expect("render surface should resize");
        }
    }
}

impl window::Handler for RenderWindow {
    fn ready(&mut self, win: &mut window::Ready<'_>) -> window::Result<()> {
        self.attach_surface(win.handle()?, win.metrics());
        win.draw();
        Ok(())
    }

    fn resize(&mut self, win: &mut window::Resize<'_>) -> window::Result<()> {
        self.resize_surface(win.metrics());
        win.draw();
        Ok(())
    }

    fn close(&mut self, close: &mut window::Close<'_>) -> window::Result<()> {
        close.close();
        Ok(())
    }

    fn closed(&mut self, closed: &mut window::Closed<'_>) -> window::Result<()> {
        closed.exit();
        Ok(())
    }

    fn draw(&mut self, _frame: &mut window::Frame<'_>) -> window::Result<()> {
        let Some(surface) = &mut self.surface else {
            return Ok(());
        };

        let mut scene = Scene::new();
        scene
            .shadow(
                Rect::new(46.0, 46.0, 220.0, 120.0),
                Shadow::new(
                    Point::new(0.0, 12.0),
                    24.0,
                    0.0,
                    Color::rgba(0.0, 0.0, 0.0, 0.38),
                ),
            )
            .fill(
                surgeist::render::Shape::rounded_rect(
                    Rect::new(40.0, 40.0, 220.0, 120.0),
                    Radii::all(18.0),
                ),
                Color::rgba(0.96, 0.97, 1.0, 1.0),
            )
            .stroke(
                surgeist::render::Shape::rounded_rect(
                    Rect::new(40.0, 40.0, 220.0, 120.0),
                    Radii::all(18.0),
                ),
                Stroke::new(2.0),
                Color::rgba(0.15, 0.33, 0.64, 1.0),
            )
            .clip(Rect::new(300.0, 48.0, 180.0, 96.0), |scene| {
                scene.layer(
                    Layer {
                        opacity: 0.75,
                        ..Layer::new()
                    },
                    |scene| {
                        scene.fill(
                            Rect::new(280.0, 28.0, 220.0, 140.0),
                            Color::rgba(1.0, 0.68, 0.26, 1.0),
                        );
                    },
                );
            });

        self.renderer
            .render(surface, &scene, Parameters::default())
            .expect("scene should render");
        Ok(())
    }
}

fn main() -> window::Result<()> {
    let renderer =
        pollster::block_on(Renderer::new(Options::default())).expect("renderer should initialize");
    window::app(RenderWindow::new(renderer))
        .open(
            window::open("render-window")
                .title("Surgeist Render Window")
                .size(window::size(640.0, 420.0)),
        )
        .run()
}
