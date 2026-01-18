use std::any::TypeId;

use hecs::{TypeIdMap, World};

use crate::{Ecs, resource::Resources};

#[derive(Debug, Default, Clone, derive_more::Deref, derive_more::DerefMut)]
pub struct Systems {
    systems: TypeIdMap<im::Vector<fn(&mut World)>>,
}

pub struct DefaultSchedule;

pub trait SystemsExt: Ecs {
    fn add_system<S: 'static>(&mut self, system: fn(&mut World)) {
        let mut systems = self.get_or_insert_resource_with::<&mut Systems>(Systems::default);
        let schedule = systems.entry(TypeId::of::<S>()).or_default();
        schedule.push_back(system);
    }
    fn run_systems<S: 'static>(&mut self) {
        let systems = {
            let systems = self.get_or_insert_resource_with::<&Systems>(Systems::default);
            systems.systems.clone()
        };
        let Some(schedule) = systems.get(&TypeId::of::<S>()) else {
            return;
        };
        for system in schedule {
            system(self);
        }
    }
}

impl<T: Ecs> SystemsExt for T {}
