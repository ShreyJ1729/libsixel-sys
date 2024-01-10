// include generated bindings
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

// re-export the generated bindings
pub use bindings::*;
