(function() {
    var implementors = Object.fromEntries([["rustix",[["impl PublicFlags for <a class=\"struct\" href=\"rustix/event/epoll/struct.CreateFlags.html\" title=\"struct rustix::event::epoll::CreateFlags\">CreateFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/event/epoll/struct.EventFlags.html\" title=\"struct rustix::event::epoll::EventFlags\">EventFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/event/struct.EventfdFlags.html\" title=\"struct rustix::event::EventfdFlags\">EventfdFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/event/struct.PollFlags.html\" title=\"struct rustix::event::PollFlags\">PollFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/fs/inotify/struct.CreateFlags.html\" title=\"struct rustix::fs::inotify::CreateFlags\">CreateFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/fs/inotify/struct.ReadFlags.html\" title=\"struct rustix::fs::inotify::ReadFlags\">ReadFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/fs/inotify/struct.WatchFlags.html\" title=\"struct rustix::fs::inotify::WatchFlags\">WatchFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/fs/struct.Access.html\" title=\"struct rustix::fs::Access\">Access</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/fs/struct.AtFlags.html\" title=\"struct rustix::fs::AtFlags\">AtFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/fs/struct.FallocateFlags.html\" title=\"struct rustix::fs::FallocateFlags\">FallocateFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/fs/struct.IFlags.html\" title=\"struct rustix::fs::IFlags\">IFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/fs/struct.MemfdFlags.html\" title=\"struct rustix::fs::MemfdFlags\">MemfdFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/fs/struct.Mode.html\" title=\"struct rustix::fs::Mode\">Mode</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/fs/struct.OFlags.html\" title=\"struct rustix::fs::OFlags\">OFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/fs/struct.RenameFlags.html\" title=\"struct rustix::fs::RenameFlags\">RenameFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/fs/struct.ResolveFlags.html\" title=\"struct rustix::fs::ResolveFlags\">ResolveFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/fs/struct.SealFlags.html\" title=\"struct rustix::fs::SealFlags\">SealFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/fs/struct.StatVfsMountFlags.html\" title=\"struct rustix::fs::StatVfsMountFlags\">StatVfsMountFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/fs/struct.StatxFlags.html\" title=\"struct rustix::fs::StatxFlags\">StatxFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/fs/struct.XattrFlags.html\" title=\"struct rustix::fs::XattrFlags\">XattrFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/io/struct.DupFlags.html\" title=\"struct rustix::io::DupFlags\">DupFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/io/struct.FdFlags.html\" title=\"struct rustix::io::FdFlags\">FdFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/io/struct.ReadWriteFlags.html\" title=\"struct rustix::io::ReadWriteFlags\">ReadWriteFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/net/struct.RecvFlags.html\" title=\"struct rustix::net::RecvFlags\">RecvFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/net/struct.SendFlags.html\" title=\"struct rustix::net::SendFlags\">SendFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/net/struct.SocketFlags.html\" title=\"struct rustix::net::SocketFlags\">SocketFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/net/xdp/struct.SockaddrXdpFlags.html\" title=\"struct rustix::net::xdp::SockaddrXdpFlags\">SockaddrXdpFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/net/xdp/struct.XdpDescOptions.html\" title=\"struct rustix::net::xdp::XdpDescOptions\">XdpDescOptions</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/net/xdp/struct.XdpOptionsFlags.html\" title=\"struct rustix::net::xdp::XdpOptionsFlags\">XdpOptionsFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/net/xdp/struct.XdpRingFlags.html\" title=\"struct rustix::net::xdp::XdpRingFlags\">XdpRingFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/net/xdp/struct.XdpUmemRegFlags.html\" title=\"struct rustix::net::xdp::XdpUmemRegFlags\">XdpUmemRegFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/process/struct.FloatingPointEmulationControl.html\" title=\"struct rustix::process::FloatingPointEmulationControl\">FloatingPointEmulationControl</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/process/struct.FloatingPointExceptionMode.html\" title=\"struct rustix::process::FloatingPointExceptionMode\">FloatingPointExceptionMode</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/process/struct.MembarrierQuery.html\" title=\"struct rustix::process::MembarrierQuery\">MembarrierQuery</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/process/struct.PidfdFlags.html\" title=\"struct rustix::process::PidfdFlags\">PidfdFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/process/struct.PidfdGetfdFlags.html\" title=\"struct rustix::process::PidfdGetfdFlags\">PidfdGetfdFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/process/struct.SpeculationFeatureControl.html\" title=\"struct rustix::process::SpeculationFeatureControl\">SpeculationFeatureControl</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/process/struct.SpeculationFeatureState.html\" title=\"struct rustix::process::SpeculationFeatureState\">SpeculationFeatureState</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/process/struct.UnalignedAccessControl.html\" title=\"struct rustix::process::UnalignedAccessControl\">UnalignedAccessControl</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/process/struct.WaitOptions.html\" title=\"struct rustix::process::WaitOptions\">WaitOptions</a>"],["impl PublicFlags for <a class=\"struct\" href=\"rustix/process/struct.WaitidOptions.html\" title=\"struct rustix::process::WaitidOptions\">WaitidOptions</a>"]]],["wayland_client",[["impl PublicFlags for <a class=\"struct\" href=\"wayland_client/protocol/wl_data_device_manager/struct.DndAction.html\" title=\"struct wayland_client::protocol::wl_data_device_manager::DndAction\">DndAction</a>"],["impl PublicFlags for <a class=\"struct\" href=\"wayland_client/protocol/wl_output/struct.Mode.html\" title=\"struct wayland_client::protocol::wl_output::Mode\">Mode</a>"],["impl PublicFlags for <a class=\"struct\" href=\"wayland_client/protocol/wl_seat/struct.Capability.html\" title=\"struct wayland_client::protocol::wl_seat::Capability\">Capability</a>"],["impl PublicFlags for <a class=\"struct\" href=\"wayland_client/protocol/wl_shell_surface/struct.Resize.html\" title=\"struct wayland_client::protocol::wl_shell_surface::Resize\">Resize</a>"],["impl PublicFlags for <a class=\"struct\" href=\"wayland_client/protocol/wl_shell_surface/struct.Transient.html\" title=\"struct wayland_client::protocol::wl_shell_surface::Transient\">Transient</a>"]]],["wayland_protocols",[["impl PublicFlags for <a class=\"struct\" href=\"wayland_protocols/wp/linux_dmabuf/zv1/client/zwp_linux_buffer_params_v1/struct.Flags.html\" title=\"struct wayland_protocols::wp::linux_dmabuf::zv1::client::zwp_linux_buffer_params_v1::Flags\">Flags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"wayland_protocols/wp/linux_dmabuf/zv1/client/zwp_linux_dmabuf_feedback_v1/struct.TrancheFlags.html\" title=\"struct wayland_protocols::wp::linux_dmabuf::zv1::client::zwp_linux_dmabuf_feedback_v1::TrancheFlags\">TrancheFlags</a>"],["impl PublicFlags for <a class=\"struct\" href=\"wayland_protocols/wp/presentation_time/client/wp_presentation_feedback/struct.Kind.html\" title=\"struct wayland_protocols::wp::presentation_time::client::wp_presentation_feedback::Kind\">Kind</a>"],["impl PublicFlags for <a class=\"struct\" href=\"wayland_protocols/xdg/shell/client/xdg_positioner/struct.ConstraintAdjustment.html\" title=\"struct wayland_protocols::xdg::shell::client::xdg_positioner::ConstraintAdjustment\">ConstraintAdjustment</a>"]]],["wayland_protocols_wlr",[["impl PublicFlags for <a class=\"struct\" href=\"wayland_protocols_wlr/layer_shell/v1/client/zwlr_layer_surface_v1/struct.Anchor.html\" title=\"struct wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::Anchor\">Anchor</a>"],["impl PublicFlags for <a class=\"struct\" href=\"wayland_protocols_wlr/screencopy/v1/client/zwlr_screencopy_frame_v1/struct.Flags.html\" title=\"struct wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_frame_v1::Flags\">Flags</a>"]]]]);
    if (window.register_implementors) {
        window.register_implementors(implementors);
    } else {
        window.pending_implementors = implementors;
    }
})()
//{"start":57,"fragment_lengths":[6405,990,1063,519]}