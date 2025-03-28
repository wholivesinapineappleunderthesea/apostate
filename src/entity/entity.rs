use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    rc::{Rc, Weak},
};

use bitflags::bitflags;

// Trait for base component info
pub trait Component: Any {
    fn as_any(&self) -> &dyn Any;
    fn get_component_type_name(&self) -> String;
}

// Trait for components stored in Entity (inside RefCell)
pub trait ComponentCell {
    fn as_any(&self) -> &dyn Any;
    fn get_component_type_name(&self) -> String;
}

// Trait to get the type name
pub trait ComponentBaseTrait {
    fn get_component_type_name(&self) -> String;
}

// Component wrapper using RefCell for interior mutability
pub struct ComponentWrapper<T: Component> {
    inner: RefCell<T>,
}

impl<T: Component> ComponentWrapper<T> {
    pub fn new(component: T) -> Self {
        Self {
            inner: RefCell::new(component),
        }
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        self.inner.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}

impl<T: Component + 'static> ComponentCell for ComponentWrapper<T> {
    fn as_any(&self) -> &dyn Any {
        &self.inner
    }

    fn get_component_type_name(&self) -> String {
        // Borrow the RefCell and clone the string to return an owned value
        let borrowed = self.inner.borrow();
        borrowed.get_component_type_name().to_string()
    }
}

// Blanket impl for all Component types
impl<T: Any + ComponentBaseTrait + 'static> Component for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_component_type_name(&self) -> String {
        self.get_component_type_name()
    }
}

bitflags! {
    pub struct EntityFlags: u32 {
        const NONE = 0b0000_0000;
        const ACTIVE = 0b0000_0001;
    }
}

pub struct Entity {
    pub name: Option<String>,
    pub flags: EntityFlags,
    pub components: Vec<Box<dyn ComponentCell>>,
    pub world: Weak<RefCell<World>>,
}

impl Entity {
    pub fn add_component<T: Component + 'static>(&mut self, component: T) {
        self.components.push(Box::new(ComponentWrapper::new(component)));
    }

    pub fn get_component<T: Component + 'static>(&self) -> Option<Ref<'_, T>> {
        for component in &self.components {
            if let Some(cell) = component.as_any().downcast_ref::<RefCell<T>>() {
                return Some(cell.borrow());
            }
        }
        None
    }

    pub fn get_component_mut<T: Component + 'static>(&self) -> Option<RefMut<'_, T>> {
        for component in &self.components {
            if let Some(cell) = component.as_any().downcast_ref::<RefCell<T>>() {
                return Some(cell.borrow_mut());
            }
        }
        None
    }

    pub fn component_count(&self) -> usize {
        self.components.len()
    }
}

pub struct World {
    pub self_ref: Weak<RefCell<World>>,
    pub entities: Vec<Rc<RefCell<Entity>>>,
}

impl World {
    pub fn new() -> Rc<RefCell<World>> {
        let world = Rc::new(RefCell::new(World {
            self_ref: Weak::new(),
            entities: Vec::new(),
        }));
        world.borrow_mut().self_ref = Rc::downgrade(&world);
        world
    }

    pub fn new_entity(&mut self) -> Rc<RefCell<Entity>> {
        let entity = Rc::new(RefCell::new(Entity {
            name: None,
            flags: EntityFlags::ACTIVE,
            components: Vec::new(),
            world: self.self_ref.clone(),
        }));
        self.entities.push(entity.clone());
        entity
    }

    pub fn destroy_entity(&mut self, entity: &Rc<RefCell<Entity>>) {
        entity.borrow_mut().flags.remove(EntityFlags::ACTIVE);
        let ptr = Rc::as_ptr(entity);
        self.entities.retain(|e| Rc::as_ptr(e) != ptr);
    }
}

// -----------------------------
// Example Components
// -----------------------------

pub struct Health {
    pub hp: i32,
}

impl ComponentBaseTrait for Health {
    fn get_component_type_name(&self) -> String {
        "Health".to_string()
    }
}

pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl ComponentBaseTrait for Position {
    fn get_component_type_name(&self) -> String {
        "Position".to_string()
    }
}