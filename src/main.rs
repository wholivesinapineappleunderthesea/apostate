

mod gfx;
use gfx::gfxrenderer::GfxRenderer;

mod entity;

async fn run()
{
    let mut gfx = GfxRenderer::new().await;
    while gfx.poll() {
        gfx.frame();
    }
}

fn main() {
    let worldrw= entity::entity::World::new();
    let mut world = worldrw.write().unwrap();
    let entity = world.new_entity();

    entity.add_component(entity::entity::Position
    {
        x:44.0,
        y:21.0
    }
    );

    // iterate over entities
    for entity in &world.entities
    {
        println!("Entity has {} components", entity.component_count());
        let position = entity.get_component::<entity::entity::Position>();
        if let Some(position) = position
        {
            println!("Position: x: {}, y: {}", position.x, position.y);
        }
    }

    println!("World has {} entities", world.entities.len());

    pollster::block_on(run());
}
