/*!
# SixtyFPS

This create is the main entry point for project using SixtyFPS UI in rust.

## How to use:

There are two ways to use this crate.

 - The `.60` code inline in a macro.
 - The `.60` code in external files compiled with `build.rs`

### The .60 code in a macro

This is the simpler way, just put the

```rust
sixtyfps::sixtyfps!{
    HelloWolrd := Text { text: "hello world"; }
}
fn main() {
#   return; // Don't run a window in an example
    HelloWolrd::default().run()
}
```

### The .60 file in external files compiled with `build.rs`

In your Cargo.toml:

FIXME! set the version

```toml
[package]
...
build = "build.rs"

[dependencies]
sixtyfps = "*"
...

[build-dependencies]
sixtyfps-build = "*"
```

In the `build.rs` file:

```ignore
fn main() {
    sixtyfps_build::compile("ui/hello.60");
}
```

Then in your main file

```ignore
sixtyfps::include_modules!();
fn main() {
    HelloWolrd::default().run()
}
```
*/

#![warn(missing_docs)]

pub use sixtyfps_rs_macro::sixtyfps;

/// internal re_exports used by the macro generated
#[doc(hidden)]
pub mod re_exports {
    pub use const_field_offset::{self, FieldOffsets};
    pub use corelib::abi::datastructures::{Component, ComponentTO, ComponentVTable, ItemTreeNode};
    pub use corelib::abi::primitives::*;
    pub use corelib::abi::properties::Property;
    pub use corelib::abi::signals::Signal;
    pub use corelib::ComponentVTable_static;
    pub use corelib::EvaluationContext;
    pub use corelib::SharedString;
    pub use once_cell::sync::Lazy;
    pub use sixtyfps_rendering_backend_gl::sixtyfps_runtime_run_component_with_gl_renderer;
    pub use vtable::{self, *};
}

#[cfg(doctest)]
mod compile_fail_tests;

/// Include the code generated with the sixtyfps-build crate from the build script
#[macro_export]
macro_rules! include_modules {
    () => {
        include!(env!("SIXTYFPS_INCLUDE_GENERATED"));
    };
}
