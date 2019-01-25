pub use generated::client;

mod generated {
    #![allow(dead_code, non_camel_case_types, unused_unsafe, unused_variables)]
    #![allow(non_upper_case_globals, non_snake_case, unused_imports)]

    #[cfg(feature = "native_lib")]
    pub mod c_interfaces {
        pub(crate) use wayland_client::sys::protocol_interfaces::wl_seat_interface;
        include!(concat!(env!("OUT_DIR"), "/gtk_primary_selection_interfaces.rs"));
    }
    #[cfg(feature = "native_lib")]
    pub mod client {
        pub(crate) use wayland_client::protocol::wl_seat;
        pub(crate) use wayland_client::{
            sys, AnonymousObject, HandledBy, NewProxy, Proxy, ProxyMap,
        };
        pub(crate) use wayland_commons::map::{Object, ObjectMetadata};
        pub(crate) use wayland_commons::wire::{Argument, ArgumentType, Message, MessageDesc};
        pub(crate) use wayland_commons::{Interface, MessageGroup};
        include!(concat!(env!("OUT_DIR"), "/gtk_primary_selection_api.rs"));
    }
    #[cfg(not(feature = "native_lib"))]
    pub mod client {
        pub(crate) use wayland_client::protocol::wl_seat;
        pub(crate) use wayland_client::{AnonymousObject, HandledBy, NewProxy, Proxy, ProxyMap};
        pub(crate) use wayland_commons::map::{Object, ObjectMetadata};
        pub(crate) use wayland_commons::wire::{Argument, ArgumentType, Message, MessageDesc};
        pub(crate) use wayland_commons::{Interface, MessageGroup};
        include!(concat!(env!("OUT_DIR"), "/gtk_primary_selection_api.rs"));
    }
}
