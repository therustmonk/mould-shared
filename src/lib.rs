//! Mould plugin for access to a shared JSON object.

#[macro_use]
extern crate mould;
extern crate permission;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
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
            Box::new(AttachWorker::new(self.objects.clone()))
        } else if request.action == "update-object" {
            Box::new(AttachWorker::new(self.objects.clone()))
        } else {
            let msg = format!("Unknown action '{}' for shared service!", request.action);
            Box::new(RejectWorker::new(msg))
        }
    }
}

struct AttachWorker {
    objects: Objects,
}

impl AttachWorker {
    fn new(objects: Objects) -> Self {
        AttachWorker {
            objects: objects,
        }
    }
}

impl<T> Worker<T> for AttachWorker where T: HasPermission<SharedPermission> {

    fn prepare(&mut self, context: &mut T, mut request: Request) -> worker::Result<Shortcut> {
        if !context.has_permission(&SharedPermission::CanAttach) {
            return Err("You haven't permissions.".into());
        }
        Ok(Shortcut::Tuned)
    }

    fn realize(&mut self, _: &mut T, _: Option<Request>) -> worker::Result<Realize> {
        Ok(Realize::Done)
    }

}
