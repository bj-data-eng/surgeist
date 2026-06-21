use surgeist::window;

#[derive(Default)]
struct HelloWindow;

impl window::Handler for HelloWindow {
    fn ready(&mut self, win: &mut window::Ready<'_>) -> window::Result<()> {
        let metrics = win.metrics();
        println!(
            "ready window {} at {}x{}",
            win.id().as_u64(),
            metrics.physical_size.width,
            metrics.physical_size.height
        );
        Ok(())
    }

    fn draw(&mut self, frame: &mut window::Frame<'_>) -> window::Result<()> {
        println!(
            "draw requested for window {} at {}x{} logical",
            frame.id().as_u64(),
            frame.size().width,
            frame.size().height
        );
        frame.exit();
        Ok(())
    }
}

fn main() -> window::Result<()> {
    window::app(HelloWindow)
        .open(
            window::open("hello")
                .title("Surgeist Hello Window")
                .size(window::size(640, 420)),
        )
        .run()
}
