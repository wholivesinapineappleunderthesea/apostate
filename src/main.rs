

mod gfx;
use gfx::gfxrenderer::GfxRenderer;

async fn run()
{
    let mut gfx = GfxRenderer::new().await;
    while gfx.poll() {
        gfx.frame();
    }
}

fn main() {
    pollster::block_on(run());
}
