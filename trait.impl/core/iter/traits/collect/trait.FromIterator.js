(function() {var implementors = {
"fixedbitset":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.81.0/core/primitive.usize.html\">usize</a>&gt; for <a class=\"struct\" href=\"fixedbitset/struct.FixedBitSet.html\" title=\"struct fixedbitset::FixedBitSet\">FixedBitSet</a>"]],
"hashbrown":[["impl&lt;K, V, S, A&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.81.0/core/primitive.tuple.html\">(K, V)</a>&gt; for <a class=\"struct\" href=\"hashbrown/hash_map/struct.HashMap.html\" title=\"struct hashbrown::hash_map::HashMap\">HashMap</a>&lt;K, V, S, A&gt;<div class=\"where\">where\n    K: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/hash/trait.Hash.html\" title=\"trait core::hash::Hash\">Hash</a>,\n    S: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/hash/trait.BuildHasher.html\" title=\"trait core::hash::BuildHasher\">BuildHasher</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a>,\n    A: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a> + Allocator,</div>"],["impl&lt;T, S, A&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;T&gt; for <a class=\"struct\" href=\"hashbrown/hash_set/struct.HashSet.html\" title=\"struct hashbrown::hash_set::HashSet\">HashSet</a>&lt;T, S, A&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/hash/trait.Hash.html\" title=\"trait core::hash::Hash\">Hash</a>,\n    S: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/hash/trait.BuildHasher.html\" title=\"trait core::hash::BuildHasher\">BuildHasher</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a>,\n    A: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a> + Allocator,</div>"]],
"indexmap":[["impl&lt;K, V, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.81.0/std/primitive.tuple.html\">(K, V)</a>&gt; for <a class=\"struct\" href=\"indexmap/map/struct.IndexMap.html\" title=\"struct indexmap::map::IndexMap\">IndexMap</a>&lt;K, V, S&gt;<div class=\"where\">where\n    K: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/hash/trait.Hash.html\" title=\"trait core::hash::Hash\">Hash</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a>,\n    S: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/hash/trait.BuildHasher.html\" title=\"trait core::hash::BuildHasher\">BuildHasher</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a>,</div>"],["impl&lt;T, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;T&gt; for <a class=\"struct\" href=\"indexmap/set/struct.IndexSet.html\" title=\"struct indexmap::set::IndexSet\">IndexSet</a>&lt;T, S&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/hash/trait.Hash.html\" title=\"trait core::hash::Hash\">Hash</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a>,\n    S: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/hash/trait.BuildHasher.html\" title=\"trait core::hash::BuildHasher\">BuildHasher</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a>,</div>"]],
"petgraph":[["impl&lt;N, E, Ty, Item, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;Item&gt; for <a class=\"struct\" href=\"petgraph/graphmap/struct.GraphMap.html\" title=\"struct petgraph::graphmap::GraphMap\">GraphMap</a>&lt;N, E, Ty, S&gt;<div class=\"where\">where\n    Item: <a class=\"trait\" href=\"petgraph/trait.IntoWeightedEdge.html\" title=\"trait petgraph::IntoWeightedEdge\">IntoWeightedEdge</a>&lt;E, NodeId = N&gt;,\n    N: <a class=\"trait\" href=\"petgraph/graphmap/trait.NodeTrait.html\" title=\"trait petgraph::graphmap::NodeTrait\">NodeTrait</a>,\n    Ty: <a class=\"trait\" href=\"petgraph/trait.EdgeType.html\" title=\"trait petgraph::EdgeType\">EdgeType</a>,\n    S: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/hash/trait.BuildHasher.html\" title=\"trait core::hash::BuildHasher\">BuildHasher</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a>,</div>"]],
"proc_macro2":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"enum\" href=\"proc_macro2/enum.TokenTree.html\" title=\"enum proc_macro2::TokenTree\">TokenTree</a>&gt; for <a class=\"struct\" href=\"proc_macro2/struct.TokenStream.html\" title=\"struct proc_macro2::TokenStream\">TokenStream</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"proc_macro2/struct.TokenStream.html\" title=\"struct proc_macro2::TokenStream\">TokenStream</a>&gt; for <a class=\"struct\" href=\"proc_macro2/struct.TokenStream.html\" title=\"struct proc_macro2::TokenStream\">TokenStream</a>"]],
"rustix":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/event/epoll/struct.CreateFlags.html\" title=\"struct rustix::event::epoll::CreateFlags\">CreateFlags</a>&gt; for <a class=\"struct\" href=\"rustix/event/epoll/struct.CreateFlags.html\" title=\"struct rustix::event::epoll::CreateFlags\">CreateFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/event/epoll/struct.EventFlags.html\" title=\"struct rustix::event::epoll::EventFlags\">EventFlags</a>&gt; for <a class=\"struct\" href=\"rustix/event/epoll/struct.EventFlags.html\" title=\"struct rustix::event::epoll::EventFlags\">EventFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/event/struct.EventfdFlags.html\" title=\"struct rustix::event::EventfdFlags\">EventfdFlags</a>&gt; for <a class=\"struct\" href=\"rustix/event/struct.EventfdFlags.html\" title=\"struct rustix::event::EventfdFlags\">EventfdFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/event/struct.PollFlags.html\" title=\"struct rustix::event::PollFlags\">PollFlags</a>&gt; for <a class=\"struct\" href=\"rustix/event/struct.PollFlags.html\" title=\"struct rustix::event::PollFlags\">PollFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/fs/inotify/struct.CreateFlags.html\" title=\"struct rustix::fs::inotify::CreateFlags\">CreateFlags</a>&gt; for <a class=\"struct\" href=\"rustix/fs/inotify/struct.CreateFlags.html\" title=\"struct rustix::fs::inotify::CreateFlags\">CreateFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/fs/inotify/struct.ReadFlags.html\" title=\"struct rustix::fs::inotify::ReadFlags\">ReadFlags</a>&gt; for <a class=\"struct\" href=\"rustix/fs/inotify/struct.ReadFlags.html\" title=\"struct rustix::fs::inotify::ReadFlags\">ReadFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/fs/inotify/struct.WatchFlags.html\" title=\"struct rustix::fs::inotify::WatchFlags\">WatchFlags</a>&gt; for <a class=\"struct\" href=\"rustix/fs/inotify/struct.WatchFlags.html\" title=\"struct rustix::fs::inotify::WatchFlags\">WatchFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/fs/struct.Access.html\" title=\"struct rustix::fs::Access\">Access</a>&gt; for <a class=\"struct\" href=\"rustix/fs/struct.Access.html\" title=\"struct rustix::fs::Access\">Access</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/fs/struct.AtFlags.html\" title=\"struct rustix::fs::AtFlags\">AtFlags</a>&gt; for <a class=\"struct\" href=\"rustix/fs/struct.AtFlags.html\" title=\"struct rustix::fs::AtFlags\">AtFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/fs/struct.FallocateFlags.html\" title=\"struct rustix::fs::FallocateFlags\">FallocateFlags</a>&gt; for <a class=\"struct\" href=\"rustix/fs/struct.FallocateFlags.html\" title=\"struct rustix::fs::FallocateFlags\">FallocateFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/fs/struct.IFlags.html\" title=\"struct rustix::fs::IFlags\">IFlags</a>&gt; for <a class=\"struct\" href=\"rustix/fs/struct.IFlags.html\" title=\"struct rustix::fs::IFlags\">IFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/fs/struct.MemfdFlags.html\" title=\"struct rustix::fs::MemfdFlags\">MemfdFlags</a>&gt; for <a class=\"struct\" href=\"rustix/fs/struct.MemfdFlags.html\" title=\"struct rustix::fs::MemfdFlags\">MemfdFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/fs/struct.Mode.html\" title=\"struct rustix::fs::Mode\">Mode</a>&gt; for <a class=\"struct\" href=\"rustix/fs/struct.Mode.html\" title=\"struct rustix::fs::Mode\">Mode</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/fs/struct.OFlags.html\" title=\"struct rustix::fs::OFlags\">OFlags</a>&gt; for <a class=\"struct\" href=\"rustix/fs/struct.OFlags.html\" title=\"struct rustix::fs::OFlags\">OFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/fs/struct.RenameFlags.html\" title=\"struct rustix::fs::RenameFlags\">RenameFlags</a>&gt; for <a class=\"struct\" href=\"rustix/fs/struct.RenameFlags.html\" title=\"struct rustix::fs::RenameFlags\">RenameFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/fs/struct.ResolveFlags.html\" title=\"struct rustix::fs::ResolveFlags\">ResolveFlags</a>&gt; for <a class=\"struct\" href=\"rustix/fs/struct.ResolveFlags.html\" title=\"struct rustix::fs::ResolveFlags\">ResolveFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/fs/struct.SealFlags.html\" title=\"struct rustix::fs::SealFlags\">SealFlags</a>&gt; for <a class=\"struct\" href=\"rustix/fs/struct.SealFlags.html\" title=\"struct rustix::fs::SealFlags\">SealFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/fs/struct.StatVfsMountFlags.html\" title=\"struct rustix::fs::StatVfsMountFlags\">StatVfsMountFlags</a>&gt; for <a class=\"struct\" href=\"rustix/fs/struct.StatVfsMountFlags.html\" title=\"struct rustix::fs::StatVfsMountFlags\">StatVfsMountFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/fs/struct.StatxFlags.html\" title=\"struct rustix::fs::StatxFlags\">StatxFlags</a>&gt; for <a class=\"struct\" href=\"rustix/fs/struct.StatxFlags.html\" title=\"struct rustix::fs::StatxFlags\">StatxFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/fs/struct.XattrFlags.html\" title=\"struct rustix::fs::XattrFlags\">XattrFlags</a>&gt; for <a class=\"struct\" href=\"rustix/fs/struct.XattrFlags.html\" title=\"struct rustix::fs::XattrFlags\">XattrFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/io/struct.DupFlags.html\" title=\"struct rustix::io::DupFlags\">DupFlags</a>&gt; for <a class=\"struct\" href=\"rustix/io/struct.DupFlags.html\" title=\"struct rustix::io::DupFlags\">DupFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/io/struct.FdFlags.html\" title=\"struct rustix::io::FdFlags\">FdFlags</a>&gt; for <a class=\"struct\" href=\"rustix/io/struct.FdFlags.html\" title=\"struct rustix::io::FdFlags\">FdFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/io/struct.ReadWriteFlags.html\" title=\"struct rustix::io::ReadWriteFlags\">ReadWriteFlags</a>&gt; for <a class=\"struct\" href=\"rustix/io/struct.ReadWriteFlags.html\" title=\"struct rustix::io::ReadWriteFlags\">ReadWriteFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/net/struct.RecvFlags.html\" title=\"struct rustix::net::RecvFlags\">RecvFlags</a>&gt; for <a class=\"struct\" href=\"rustix/net/struct.RecvFlags.html\" title=\"struct rustix::net::RecvFlags\">RecvFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/net/struct.SendFlags.html\" title=\"struct rustix::net::SendFlags\">SendFlags</a>&gt; for <a class=\"struct\" href=\"rustix/net/struct.SendFlags.html\" title=\"struct rustix::net::SendFlags\">SendFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/net/struct.SocketFlags.html\" title=\"struct rustix::net::SocketFlags\">SocketFlags</a>&gt; for <a class=\"struct\" href=\"rustix/net/struct.SocketFlags.html\" title=\"struct rustix::net::SocketFlags\">SocketFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/net/xdp/struct.SockaddrXdpFlags.html\" title=\"struct rustix::net::xdp::SockaddrXdpFlags\">SockaddrXdpFlags</a>&gt; for <a class=\"struct\" href=\"rustix/net/xdp/struct.SockaddrXdpFlags.html\" title=\"struct rustix::net::xdp::SockaddrXdpFlags\">SockaddrXdpFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/net/xdp/struct.XdpDescOptions.html\" title=\"struct rustix::net::xdp::XdpDescOptions\">XdpDescOptions</a>&gt; for <a class=\"struct\" href=\"rustix/net/xdp/struct.XdpDescOptions.html\" title=\"struct rustix::net::xdp::XdpDescOptions\">XdpDescOptions</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/net/xdp/struct.XdpOptionsFlags.html\" title=\"struct rustix::net::xdp::XdpOptionsFlags\">XdpOptionsFlags</a>&gt; for <a class=\"struct\" href=\"rustix/net/xdp/struct.XdpOptionsFlags.html\" title=\"struct rustix::net::xdp::XdpOptionsFlags\">XdpOptionsFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/net/xdp/struct.XdpRingFlags.html\" title=\"struct rustix::net::xdp::XdpRingFlags\">XdpRingFlags</a>&gt; for <a class=\"struct\" href=\"rustix/net/xdp/struct.XdpRingFlags.html\" title=\"struct rustix::net::xdp::XdpRingFlags\">XdpRingFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/net/xdp/struct.XdpUmemRegFlags.html\" title=\"struct rustix::net::xdp::XdpUmemRegFlags\">XdpUmemRegFlags</a>&gt; for <a class=\"struct\" href=\"rustix/net/xdp/struct.XdpUmemRegFlags.html\" title=\"struct rustix::net::xdp::XdpUmemRegFlags\">XdpUmemRegFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/process/struct.FloatingPointEmulationControl.html\" title=\"struct rustix::process::FloatingPointEmulationControl\">FloatingPointEmulationControl</a>&gt; for <a class=\"struct\" href=\"rustix/process/struct.FloatingPointEmulationControl.html\" title=\"struct rustix::process::FloatingPointEmulationControl\">FloatingPointEmulationControl</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/process/struct.FloatingPointExceptionMode.html\" title=\"struct rustix::process::FloatingPointExceptionMode\">FloatingPointExceptionMode</a>&gt; for <a class=\"struct\" href=\"rustix/process/struct.FloatingPointExceptionMode.html\" title=\"struct rustix::process::FloatingPointExceptionMode\">FloatingPointExceptionMode</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/process/struct.MembarrierQuery.html\" title=\"struct rustix::process::MembarrierQuery\">MembarrierQuery</a>&gt; for <a class=\"struct\" href=\"rustix/process/struct.MembarrierQuery.html\" title=\"struct rustix::process::MembarrierQuery\">MembarrierQuery</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/process/struct.PidfdFlags.html\" title=\"struct rustix::process::PidfdFlags\">PidfdFlags</a>&gt; for <a class=\"struct\" href=\"rustix/process/struct.PidfdFlags.html\" title=\"struct rustix::process::PidfdFlags\">PidfdFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/process/struct.PidfdGetfdFlags.html\" title=\"struct rustix::process::PidfdGetfdFlags\">PidfdGetfdFlags</a>&gt; for <a class=\"struct\" href=\"rustix/process/struct.PidfdGetfdFlags.html\" title=\"struct rustix::process::PidfdGetfdFlags\">PidfdGetfdFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/process/struct.SpeculationFeatureControl.html\" title=\"struct rustix::process::SpeculationFeatureControl\">SpeculationFeatureControl</a>&gt; for <a class=\"struct\" href=\"rustix/process/struct.SpeculationFeatureControl.html\" title=\"struct rustix::process::SpeculationFeatureControl\">SpeculationFeatureControl</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/process/struct.SpeculationFeatureState.html\" title=\"struct rustix::process::SpeculationFeatureState\">SpeculationFeatureState</a>&gt; for <a class=\"struct\" href=\"rustix/process/struct.SpeculationFeatureState.html\" title=\"struct rustix::process::SpeculationFeatureState\">SpeculationFeatureState</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/process/struct.UnalignedAccessControl.html\" title=\"struct rustix::process::UnalignedAccessControl\">UnalignedAccessControl</a>&gt; for <a class=\"struct\" href=\"rustix/process/struct.UnalignedAccessControl.html\" title=\"struct rustix::process::UnalignedAccessControl\">UnalignedAccessControl</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/process/struct.WaitOptions.html\" title=\"struct rustix::process::WaitOptions\">WaitOptions</a>&gt; for <a class=\"struct\" href=\"rustix/process/struct.WaitOptions.html\" title=\"struct rustix::process::WaitOptions\">WaitOptions</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"rustix/process/struct.WaitidOptions.html\" title=\"struct rustix::process::WaitidOptions\">WaitidOptions</a>&gt; for <a class=\"struct\" href=\"rustix/process/struct.WaitidOptions.html\" title=\"struct rustix::process::WaitidOptions\">WaitidOptions</a>"]],
"smallvec":[["impl&lt;A: <a class=\"trait\" href=\"smallvec/trait.Array.html\" title=\"trait smallvec::Array\">Array</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;&lt;A as <a class=\"trait\" href=\"smallvec/trait.Array.html\" title=\"trait smallvec::Array\">Array</a>&gt;::<a class=\"associatedtype\" href=\"smallvec/trait.Array.html#associatedtype.Item\" title=\"type smallvec::Array::Item\">Item</a>&gt; for <a class=\"struct\" href=\"smallvec/struct.SmallVec.html\" title=\"struct smallvec::SmallVec\">SmallVec</a>&lt;A&gt;"]],
"syn":[["impl&lt;T, P&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"enum\" href=\"syn/punctuated/enum.Pair.html\" title=\"enum syn::punctuated::Pair\">Pair</a>&lt;T, P&gt;&gt; for <a class=\"struct\" href=\"syn/punctuated/struct.Punctuated.html\" title=\"struct syn::punctuated::Punctuated\">Punctuated</a>&lt;T, P&gt;"],["impl&lt;T, P&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;T&gt; for <a class=\"struct\" href=\"syn/punctuated/struct.Punctuated.html\" title=\"struct syn::punctuated::Punctuated\">Punctuated</a>&lt;T, P&gt;<div class=\"where\">where\n    P: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a>,</div>"]],
"wayland_client":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"wayland_client/protocol/wl_data_device_manager/struct.DndAction.html\" title=\"struct wayland_client::protocol::wl_data_device_manager::DndAction\">DndAction</a>&gt; for <a class=\"struct\" href=\"wayland_client/protocol/wl_data_device_manager/struct.DndAction.html\" title=\"struct wayland_client::protocol::wl_data_device_manager::DndAction\">DndAction</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"wayland_client/protocol/wl_output/struct.Mode.html\" title=\"struct wayland_client::protocol::wl_output::Mode\">Mode</a>&gt; for <a class=\"struct\" href=\"wayland_client/protocol/wl_output/struct.Mode.html\" title=\"struct wayland_client::protocol::wl_output::Mode\">Mode</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"wayland_client/protocol/wl_seat/struct.Capability.html\" title=\"struct wayland_client::protocol::wl_seat::Capability\">Capability</a>&gt; for <a class=\"struct\" href=\"wayland_client/protocol/wl_seat/struct.Capability.html\" title=\"struct wayland_client::protocol::wl_seat::Capability\">Capability</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"wayland_client/protocol/wl_shell_surface/struct.Resize.html\" title=\"struct wayland_client::protocol::wl_shell_surface::Resize\">Resize</a>&gt; for <a class=\"struct\" href=\"wayland_client/protocol/wl_shell_surface/struct.Resize.html\" title=\"struct wayland_client::protocol::wl_shell_surface::Resize\">Resize</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"wayland_client/protocol/wl_shell_surface/struct.Transient.html\" title=\"struct wayland_client::protocol::wl_shell_surface::Transient\">Transient</a>&gt; for <a class=\"struct\" href=\"wayland_client/protocol/wl_shell_surface/struct.Transient.html\" title=\"struct wayland_client::protocol::wl_shell_surface::Transient\">Transient</a>"]],
"wayland_protocols":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"wayland_protocols/wp/linux_dmabuf/zv1/client/zwp_linux_buffer_params_v1/struct.Flags.html\" title=\"struct wayland_protocols::wp::linux_dmabuf::zv1::client::zwp_linux_buffer_params_v1::Flags\">Flags</a>&gt; for <a class=\"struct\" href=\"wayland_protocols/wp/linux_dmabuf/zv1/client/zwp_linux_buffer_params_v1/struct.Flags.html\" title=\"struct wayland_protocols::wp::linux_dmabuf::zv1::client::zwp_linux_buffer_params_v1::Flags\">Flags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"wayland_protocols/wp/linux_dmabuf/zv1/client/zwp_linux_dmabuf_feedback_v1/struct.TrancheFlags.html\" title=\"struct wayland_protocols::wp::linux_dmabuf::zv1::client::zwp_linux_dmabuf_feedback_v1::TrancheFlags\">TrancheFlags</a>&gt; for <a class=\"struct\" href=\"wayland_protocols/wp/linux_dmabuf/zv1/client/zwp_linux_dmabuf_feedback_v1/struct.TrancheFlags.html\" title=\"struct wayland_protocols::wp::linux_dmabuf::zv1::client::zwp_linux_dmabuf_feedback_v1::TrancheFlags\">TrancheFlags</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"wayland_protocols/wp/presentation_time/client/wp_presentation_feedback/struct.Kind.html\" title=\"struct wayland_protocols::wp::presentation_time::client::wp_presentation_feedback::Kind\">Kind</a>&gt; for <a class=\"struct\" href=\"wayland_protocols/wp/presentation_time/client/wp_presentation_feedback/struct.Kind.html\" title=\"struct wayland_protocols::wp::presentation_time::client::wp_presentation_feedback::Kind\">Kind</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"wayland_protocols/xdg/shell/client/xdg_positioner/struct.ConstraintAdjustment.html\" title=\"struct wayland_protocols::xdg::shell::client::xdg_positioner::ConstraintAdjustment\">ConstraintAdjustment</a>&gt; for <a class=\"struct\" href=\"wayland_protocols/xdg/shell/client/xdg_positioner/struct.ConstraintAdjustment.html\" title=\"struct wayland_protocols::xdg::shell::client::xdg_positioner::ConstraintAdjustment\">ConstraintAdjustment</a>"]],
"wayland_protocols_wlr":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"wayland_protocols_wlr/layer_shell/v1/client/zwlr_layer_surface_v1/struct.Anchor.html\" title=\"struct wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::Anchor\">Anchor</a>&gt; for <a class=\"struct\" href=\"wayland_protocols_wlr/layer_shell/v1/client/zwlr_layer_surface_v1/struct.Anchor.html\" title=\"struct wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::Anchor\">Anchor</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;<a class=\"struct\" href=\"wayland_protocols_wlr/screencopy/v1/client/zwlr_screencopy_frame_v1/struct.Flags.html\" title=\"struct wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_frame_v1::Flags\">Flags</a>&gt; for <a class=\"struct\" href=\"wayland_protocols_wlr/screencopy/v1/client/zwlr_screencopy_frame_v1/struct.Flags.html\" title=\"struct wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_frame_v1::Flags\">Flags</a>"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()