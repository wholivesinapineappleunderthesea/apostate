

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
    let worldrc= entity::entity::World::new();
    let mut world = worldrc.borrow_mut();
    let newentity = world.new_entity();

    newentity.borrow_mut().add_component(entity::entity::Position
    {
        x:44.0,
        y:21.0
    }
    );

    // iterate over entities
    for entity in world.entities.iter()
    {
        let entity = entity.borrow();
        println!("Entity: {:?}", entity.name);
        println!("Entity flags: {:?}", entity.flags);
        println!("Entity has {} components", entity.component_count());
        let mut position = entity.get_component_mut::<entity::entity::Position>();
        if let Some(mut position) = position
        {
            println!("Position: x: {}, y: {}", position.x, position.y);
            position.x = 924983.0;
            
        }
    }

    for entity in world.entities.iter()
    {
        let entity = entity.borrow();
        println!("Entity: {:?}", entity.name);
        println!("Entity flags: {:?}", entity.flags);
        println!("Entity has {} components", entity.component_count());
        let mut position = entity.get_component_mut::<entity::entity::Position>();
        if let Some(mut position) = position
        {
            println!("Position: x: {}, y: {}", position.x, position.y);
            
        }
    }


    println!("World has {} entities", world.entities.len());

    pollster::block_on(run());
}
