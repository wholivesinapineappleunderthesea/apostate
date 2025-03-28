use std::{any::Any, collections::HashMap, sync::{Arc, Weak}};
use std::sync::{Mutex, RwLock};
use bitflags::bitflags;

pub trait Component: Any {
    fn as_any(&self) -> &dyn Any;
    fn name(&self) -> &str;
}

impl<T: Any> Component for T
where
    T: 'static + Any + ComponentBaseTrait,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        self.get_component_type_name()
    }
}

trait ComponentBaseTrait {
    fn get_component_type_name(&self) -> &str;
}

bitflags! {
    pub struct EntityFlags: u32
    {
        const NONE = 0b0000_0000;
        const ACTIVE = 0b0000_0001;
    }
}

pub struct Entity
{
    pub name: Option<String>,
    pub flags: EntityFlags,
    pub components: Vec<Box<Arc<dyn Component>>>,
    pub world: Weak<RwLock<World>>,
}

impl Entity
{
    
    pub fn add_component<T: Component + 'static>(&mut self, component: T)
    {
        self.components.push(Box::new(Arc::new(component)));
    }

    // iterates components, looks for the first one that derives from T
    pub fn get_component<T: Component + 'static>(&self) -> Option<&T>
    {
        for component in &self.components
        {
            if let Some(component) = component.as_any().downcast_ref::<T>()
            {
                return Some(component);
            }
        }
        None
    }

    pub fn component_count(&self) -> usize
    {
        self.components.len()
    }

    /*
    
    
    
    pub fn get_component_mut<T: Component + 'static>(&mut self) -> Option<&mut T>
    {
        for component in &mut self.components
        {
            if let Some(component) = component.downcast_mut::<T>()
            {
                return Some(component);
            }
        }
        None
    }
    */
}

pub struct World
{
    pub self_ref: Weak<RwLock<World>>,
    pub entities: Vec<Entity>,
}

impl World
{
    pub fn new() -> Arc<RwLock<World>>
    {
        let world = World{
            self_ref: Weak::new(),
            entities: Vec::new(),
        };
        let arc_world = Arc::new(RwLock::new(world));
        let weak = Arc::downgrade(&arc_world);
        arc_world.write().unwrap().self_ref = weak;
        arc_world
    }

    pub fn new_entity(&mut self) -> &mut Entity
    {
        let entity = Entity{
            name: None,
            flags: EntityFlags::ACTIVE,
            components: Vec::new(),
            world: self.self_ref.clone(),
        };
        self.entities.push(entity);
        self.entities.last_mut().unwrap()
    }

    pub fn destroy_entity(&mut self, entity: &mut Entity)
    {
        entity.flags.remove(EntityFlags::ACTIVE);
        self.entities.retain(|e| e as *const Entity != entity as *const Entity);
    }

}

// test components
pub struct Health {
    pub hp: i32,
}

impl ComponentBaseTrait for Health {
    fn get_component_type_name(&self) -> &str {
        "Health"
    }
}

pub struct Position {
    pub y: f32,
    pub x: f32,
}

impl ComponentBaseTrait for Position {
    fn get_component_type_name(&self) -> &str {
        "Position"
    }
}