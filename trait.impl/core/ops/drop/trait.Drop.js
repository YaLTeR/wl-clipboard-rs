(function() {var implementors = {
"hashbrown":[["impl&lt;T, A: Allocator&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"hashbrown/raw/struct.RawIntoIter.html\" title=\"struct hashbrown::raw::RawIntoIter\">RawIntoIter</a>&lt;T, A&gt;"],["impl&lt;T, A: Allocator&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"hashbrown/raw/struct.RawTable.html\" title=\"struct hashbrown::raw::RawTable\">RawTable</a>&lt;T, A&gt;"],["impl&lt;T, A: Allocator&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"hashbrown/raw/struct.RawDrain.html\" title=\"struct hashbrown::raw::RawDrain\">RawDrain</a>&lt;'_, T, A&gt;"]],
"indexmap":[["impl&lt;I, K, V, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"indexmap/map/struct.Splice.html\" title=\"struct indexmap::map::Splice\">Splice</a>&lt;'_, I, K, V, S&gt;<div class=\"where\">where\n    I: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/iter/traits/iterator/trait.Iterator.html\" title=\"trait core::iter::traits::iterator::Iterator\">Iterator</a>&lt;Item = <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.76.0/std/primitive.tuple.html\">(K, V)</a>&gt;,\n    K: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/hash/trait.Hash.html\" title=\"trait core::hash::Hash\">Hash</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a>,\n    S: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/hash/trait.BuildHasher.html\" title=\"trait core::hash::BuildHasher\">BuildHasher</a>,</div>"]],
"libloading":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"libloading/os/unix/struct.Library.html\" title=\"struct libloading::os::unix::Library\">Library</a>"]],
"once_cell":[["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"once_cell/race/struct.OnceBox.html\" title=\"struct once_cell::race::OnceBox\">OnceBox</a>&lt;T&gt;"]],
"rustix":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"rustix/net/struct.RecvAncillaryBuffer.html\" title=\"struct rustix::net::RecvAncillaryBuffer\">RecvAncillaryBuffer</a>&lt;'_&gt;"],["impl&lt;'data, T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"rustix/net/struct.AncillaryIter.html\" title=\"struct rustix::net::AncillaryIter\">AncillaryIter</a>&lt;'data, T&gt;"]],
"smallvec":[["impl&lt;'a, T: 'a + <a class=\"trait\" href=\"smallvec/trait.Array.html\" title=\"trait smallvec::Array\">Array</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"smallvec/struct.Drain.html\" title=\"struct smallvec::Drain\">Drain</a>&lt;'a, T&gt;"],["impl&lt;A: <a class=\"trait\" href=\"smallvec/trait.Array.html\" title=\"trait smallvec::Array\">Array</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"smallvec/struct.SmallVec.html\" title=\"struct smallvec::SmallVec\">SmallVec</a>&lt;A&gt;"],["impl&lt;A: <a class=\"trait\" href=\"smallvec/trait.Array.html\" title=\"trait smallvec::Array\">Array</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"smallvec/struct.IntoIter.html\" title=\"struct smallvec::IntoIter\">IntoIter</a>&lt;A&gt;"]],
"syn":[["impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"syn/parse/struct.ParseBuffer.html\" title=\"struct syn::parse::ParseBuffer\">ParseBuffer</a>&lt;'a&gt;"]],
"tempfile":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"tempfile/struct.TempPath.html\" title=\"struct tempfile::TempPath\">TempPath</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"tempfile/struct.TempDir.html\" title=\"struct tempfile::TempDir\">TempDir</a>"]],
"wayland_client":[["impl&lt;'a, State&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"wayland_client/struct.QueueFreezeGuard.html\" title=\"struct wayland_client::QueueFreezeGuard\">QueueFreezeGuard</a>&lt;'a, State&gt;"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()