var sourcesIndex = JSON.parse('{\
"bitflags":["",[],["external.rs","internal.rs","iter.rs","lib.rs","parser.rs","public.rs","traits.rs"]],\
"bytecount":["",[],["integer_simd.rs","lib.rs","naive.rs"]],\
"cfg_if":["",[],["lib.rs"]],\
"derive_new":["",[],["lib.rs"]],\
"dlib":["",[],["lib.rs"]],\
"downcast_rs":["",[],["lib.rs"]],\
"fastrand":["",[],["global_rng.rs","lib.rs"]],\
"fixedbitset":["",[],["lib.rs","range.rs"]],\
"fnv":["",[],["lib.rs"]],\
"hashbrown":["",[["external_trait_impls",[],["mod.rs"]],["raw",[],["alloc.rs","bitmask.rs","mod.rs","sse2.rs"]]],["lib.rs","macros.rs","map.rs","scopeguard.rs","set.rs"]],\
"indexmap":["",[["map",[["core",[],["raw.rs"]]],["core.rs"]]],["arbitrary.rs","equivalent.rs","lib.rs","macros.rs","map.rs","mutable_keys.rs","set.rs","util.rs"]],\
"io_lifetimes":["",[],["example_ffi.rs","lib.rs","portability.rs","raw.rs","traits.rs","views.rs"]],\
"lazy_static":["",[],["inline_lazy.rs","lib.rs"]],\
"libc":["",[["unix",[["linux_like",[["linux",[["arch",[["generic",[],["mod.rs"]]],["mod.rs"]],["gnu",[["b64",[["x86_64",[],["align.rs","mod.rs","not_x32.rs"]]],["mod.rs"]]],["align.rs","mod.rs"]]],["align.rs","mod.rs","non_exhaustive.rs"]]],["mod.rs"]]],["align.rs","mod.rs"]]],["fixed_width_ints.rs","lib.rs","macros.rs"]],\
"libloading":["",[["os",[["unix",[],["consts.rs","mod.rs"]]],["mod.rs"]]],["changelog.rs","error.rs","lib.rs","safe.rs","util.rs"]],\
"linux_raw_sys":["",[["x86_64",[],["errno.rs","general.rs","ioctl.rs"]]],["lib.rs"]],\
"log":["",[],["lib.rs","macros.rs"]],\
"memchr":["",[["memchr",[["x86",[],["avx.rs","mod.rs","sse2.rs"]]],["fallback.rs","iter.rs","mod.rs","naive.rs"]],["memmem",[["prefilter",[["x86",[],["avx.rs","mod.rs","sse.rs"]]],["fallback.rs","genericsimd.rs","mod.rs"]],["x86",[],["avx.rs","mod.rs","sse.rs"]]],["byte_frequencies.rs","genericsimd.rs","mod.rs","rabinkarp.rs","rarebytes.rs","twoway.rs","util.rs","vector.rs"]]],["cow.rs","lib.rs"]],\
"memoffset":["",[],["lib.rs","offset_of.rs","raw_field.rs","span_of.rs"]],\
"minimal_lexical":["",[],["bigint.rs","extended_float.rs","lemire.rs","lib.rs","mask.rs","num.rs","number.rs","parse.rs","rounding.rs","slow.rs","stackvec.rs","table.rs","table_lemire.rs","table_small.rs"]],\
"nix":["",[["mount",[],["linux.rs","mod.rs"]],["net",[],["if_.rs","mod.rs"]],["sys",[["ioctl",[],["linux.rs","mod.rs"]],["ptrace",[],["linux.rs","mod.rs"]],["socket",[],["addr.rs","mod.rs","sockopt.rs"]]],["aio.rs","epoll.rs","eventfd.rs","inotify.rs","memfd.rs","mman.rs","mod.rs","personality.rs","pthread.rs","quota.rs","reboot.rs","resource.rs","select.rs","sendfile.rs","signal.rs","signalfd.rs","stat.rs","statfs.rs","statvfs.rs","sysinfo.rs","termios.rs","time.rs","timer.rs","timerfd.rs","uio.rs","utsname.rs","wait.rs"]]],["dir.rs","env.rs","errno.rs","fcntl.rs","features.rs","ifaddrs.rs","kmod.rs","lib.rs","macros.rs","mqueue.rs","poll.rs","pty.rs","sched.rs","time.rs","ucontext.rs","unistd.rs"]],\
"nom":["",[["bits",[],["complete.rs","mod.rs","streaming.rs"]],["branch",[],["mod.rs"]],["bytes",[],["complete.rs","mod.rs","streaming.rs"]],["character",[],["complete.rs","mod.rs","streaming.rs"]],["combinator",[],["mod.rs"]],["multi",[],["mod.rs"]],["number",[],["complete.rs","mod.rs","streaming.rs"]],["sequence",[],["mod.rs"]]],["error.rs","internal.rs","lib.rs","macros.rs","str.rs","traits.rs"]],\
"once_cell":["",[],["imp_std.rs","lib.rs","race.rs"]],\
"os_pipe":["",[],["lib.rs","unix.rs"]],\
"petgraph":["",[["algo",[],["astar.rs","bellman_ford.rs","dijkstra.rs","dominators.rs","feedback_arc_set.rs","floyd_warshall.rs","isomorphism.rs","k_shortest_path.rs","matching.rs","mod.rs","simple_paths.rs","tred.rs"]],["graph_impl",[["stable_graph",[],["mod.rs"]]],["frozen.rs","mod.rs"]],["visit",[],["dfsvisit.rs","filter.rs","macros.rs","mod.rs","reversed.rs","traversal.rs"]]],["adj.rs","csr.rs","data.rs","dot.rs","graphmap.rs","iter_format.rs","iter_utils.rs","lib.rs","macros.rs","matrix_graph.rs","operator.rs","prelude.rs","scored.rs","traits_graph.rs","unionfind.rs","util.rs"]],\
"pin_utils":["",[],["lib.rs","projection.rs","stack_pin.rs"]],\
"proc_macro2":["",[],["detection.rs","extra.rs","fallback.rs","lib.rs","marker.rs","parse.rs","rcvec.rs","wrapper.rs"]],\
"quick_xml":["",[["events",[],["attributes.rs","mod.rs"]],["reader",[],["buffered_reader.rs","mod.rs","ns_reader.rs","parser.rs","slice_reader.rs"]]],["encoding.rs","errors.rs","escapei.rs","lib.rs","name.rs","utils.rs","writer.rs"]],\
"quote":["",[],["ext.rs","format.rs","ident_fragment.rs","lib.rs","runtime.rs","spanned.rs","to_tokens.rs"]],\
"rustix":["",[["backend",[["linux_raw",[["arch",[["asm",[],["mod.rs","x86_64.rs"]]],["mod.rs"]],["fs",[],["dir.rs","inotify.rs","makedev.rs","mod.rs","syscalls.rs","types.rs"]],["io",[],["errno.rs","mod.rs","syscalls.rs","types.rs"]],["ugid",[],["mod.rs","syscalls.rs"]]],["c.rs","conv.rs","mod.rs","reg.rs"]]]],["fs",[],["abs.rs","at.rs","constants.rs","copy_file_range.rs","cwd.rs","dir.rs","fadvise.rs","fcntl.rs","fd.rs","file_type.rs","id.rs","ioctl.rs","makedev.rs","memfd_create.rs","mod.rs","mount.rs","openat2.rs","raw_dir.rs","seek_from.rs","sendfile.rs","statx.rs","sync.rs","xattr.rs"]],["io",[],["close.rs","dup.rs","errno.rs","fcntl.rs","ioctl.rs","mod.rs","read_write.rs"]],["maybe_polyfill",[["std",[],["mod.rs"]]]],["path",[],["arg.rs","mod.rs"]]],["bitcast.rs","cstr.rs","ffi.rs","lib.rs","timespec.rs","ugid.rs","utils.rs","weak.rs"]],\
"scoped_tls":["",[],["lib.rs"]],\
"smallvec":["",[],["lib.rs"]],\
"static_assertions":["",[],["assert_cfg.rs","assert_eq_align.rs","assert_eq_size.rs","assert_fields.rs","assert_impl.rs","assert_obj_safe.rs","assert_trait.rs","assert_type.rs","const_assert.rs","lib.rs"]],\
"tempfile":["",[["file",[["imp",[],["mod.rs","unix.rs"]]],["mod.rs"]]],["dir.rs","error.rs","lib.rs","spooled.rs","util.rs"]],\
"thiserror":["",[],["aserror.rs","display.rs","lib.rs"]],\
"thiserror_impl":["",[],["ast.rs","attr.rs","expand.rs","fmt.rs","generics.rs","lib.rs","prop.rs","valid.rs"]],\
"tree_magic_mini":["",[["basetype",[],["check.rs","init.rs","mod.rs"]],["fdo_magic",[["builtin",[],["check.rs","init.rs","mod.rs","runtime.rs"]]],["check.rs","mod.rs","ruleset.rs"]]],["lib.rs"]],\
"unicode_ident":["",[],["lib.rs","tables.rs"]],\
"wayland_backend":["",[["rs",[["client_impl",[],["mod.rs"]],["server_impl",[],["client.rs","common_poll.rs","handle.rs","mod.rs","registry.rs"]]],["debug.rs","map.rs","mod.rs","socket.rs","wire.rs"]],["types",[],["client.rs","mod.rs","server.rs"]]],["client_api.rs","core_interfaces.rs","lib.rs","protocol.rs","server_api.rs"]],\
"wayland_client":["",[],["conn.rs","event_queue.rs","globals.rs","lib.rs"]],\
"wayland_protocols":["",[],["ext.rs","lib.rs","protocol_macro.rs","wp.rs","xdg.rs","xwayland.rs"]],\
"wayland_protocols_wlr":["",[],["lib.rs","protocol_macro.rs"]],\
"wayland_scanner":["",[],["c_interfaces.rs","client_gen.rs","common.rs","interfaces.rs","lib.rs","parse.rs","protocol.rs","server_gen.rs","token.rs","util.rs"]],\
"wayland_server":["",[],["client.rs","dispatch.rs","display.rs","global.rs","lib.rs","socket.rs"]],\
"wayland_sys":["",[],["client.rs","common.rs","lib.rs","server.rs"]],\
"wl_clipboard_rs":["",[],["common.rs","copy.rs","lib.rs","paste.rs","seat_data.rs","utils.rs"]]\
}');
createSourceSidebar();
