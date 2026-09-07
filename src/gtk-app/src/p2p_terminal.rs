use fm_core::rpc::FileSystemRpc;
use std::rc::Rc;

pub fn session_factory(provider: &Rc<dyn FileSystemRpc>) -> Option<crate::terminal::SessionFactory> {
    let any = provider.as_any()?;
    let inner = match any.downcast_ref::<panel_router::RoutingProvider>() {
        Some(rp) => rp.inner(),
        None => provider.clone(),
    };
    let rpc = inner
        .as_any()?
        .downcast_ref::<virtualfs::p2p_rpc::RemoteFileSystemRpc>()?;

    virtualfs::p2p_rpc::peer_terminal(&rpc.peer_id)?;
    let net_tx = rpc.net_tx.clone();
    let peer_id = rpc.peer_id.clone();

    Some(Rc::new(move || {
        p2p_runtime::peer_shell(&peer_id, net_tx.clone(), 24, 80)
            .ok_or_else(|| "this peer no longer shares a terminal".to_string())
    }))
}
