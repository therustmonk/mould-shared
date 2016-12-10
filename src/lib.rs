//! Mould plugin for access to a shared JSON object.

#[macro_use]
extern crate mould;
extern crate permission;

use std::collections::HashMap;
use std::sync::{Arc, Weak, Mutex};
use mould::prelude::*;
use mould::rustc_serialize::json::Object;
use permission::HasPermission;

pub enum SharedPermission {
    /// Attach new object
    CanAttach,
    /// Can connect, listen to updates, and send updates
    CanUpdate,
}

type Objects = Arc<Mutex<HashMap<String, Arc<Object>>>>;

pub struct SharedService {
    objects: Objects,
}

impl SharedService {

    pub fn new() -> Self {
        SharedService {
            objects: Arc::new(Mutex::new(HashMap::new())),
        }
    }

}

impl<T> Service<T> for SharedService where T: HasPermission<SharedPermission> {
    fn route(&self, request: &Request) -> Box<Worker<T>> {
        if request.action == "attach-object" {
            Box::new(UpdateWorker::new(self.objects.clone(), true))
        } else if request.action == "update-object" {
            Box::new(UpdateWorker::new(self.objects.clone(), false))
        } else {
            let msg = format!("Unknown action '{}' for shared service!", request.action);
            Box::new(RejectWorker::new(msg))
        }
    }
}

struct UpdateWorker {
    drop_name: Option<String>,
    attach: bool,
    objects: Objects,
    object: Weak<Object>,
}

impl UpdateWorker {
    fn new(objects: Objects, attach: bool) -> Self {
        UpdateWorker {
            drop_name: None,
            attach: attach,
            objects: objects,
            object: Weak::new(),
        }
    }
}

impl Drop for UpdateWorker {
    fn drop(&mut self) {
        if let Some(ref name) = self.drop_name {
            let mut objects = self.objects.lock().unwrap();
            objects.remove(name);
        }
    }
}

impl<T> Worker<T> for UpdateWorker where T: HasPermission<SharedPermission> {

    fn prepare(&mut self, context: &mut T, mut request: Request) -> worker::Result<Shortcut> {
        if !context.has_permission(&SharedPermission::CanAttach) {
            return Err("You haven't permissions.".into());
        }
        let name: String = request.extract("name").ok_or("name of shared object not set")?;
        if self.attach {
            let object: Object = request.extract("object").ok_or("object not provided")?;
            let mut objects = self.objects.lock().unwrap();
            if objects.contains_key(&name) {
                return Ok(Shortcut::Reject(format!("name '{}' already attached", name)));
            }
            let object = Arc::new(object);
            self.object = Arc::downgrade(&object);
            objects.insert(name.clone(), object);
            self.drop_name = Some(name);
        } else {
            let objects = self.objects.lock().unwrap();
            match objects.get(&name) {
                Some(object) => {
                    self.object = Arc::downgrade(&object);
                },
                None => {
                    return Ok(Shortcut::Reject(format!("name '{}' hasn't attached", name)));
                },
            }
        }
        Ok(Shortcut::Tuned)
    }

    fn realize(&mut self, _: &mut T, _: Option<Request>) -> worker::Result<Realize> {
        Ok(Realize::Done)
    }

}

