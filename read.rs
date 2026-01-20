struct GraphPopulator {
    graph: Graph,
}

pub fn add_graph_event_handlers(
    graph: Graph,
    nodes: NodeInformer,
    pods: PodInformer,
    pvs: PersistentVolumeInformer,
    attachments: VolumeAttachmentInformer,
    slices: Option<ResourceSliceInformer>,
    pcrs: Option<PodCertificateRequestInformer>,
) {
    let g = GraphPopulator {
        graph,
    };

    let pod_handler = pods.informer().add_event_handler(ResourceEventHandlerFuncs {
        add_func: Some(g.add_pod),
        update_func: Some(g.update_pod),
        delete_func: Some(g.delete_pod),
    });

    let pvs_handler = pvs.informer().add_event_handler(ResourceEventHandlerFuncs {
        add_func: Some(g.add_pv),
        update_func: Some(g.update_pv),
        delete_func: Some(g.delete_pv),
    });

    let attach_handler = attachments.informer().add_event_handler(ResourceEventHandlerFuncs {
        add_func: Some(g.add_volume_attachment),
        update_func: Some(g.update_volume_attachment),
        delete_func: Some(g.delete_volume_attachment),
    });

    let mut synced = vec![
        pod_handler.has_synced,
        pvs_handler.has_synced,
        attach_handler.has_synced,
    ];

    if let Some(slices) = slices {
        let slice_handler = slices.informer().add_event_handler(ResourceEventHandlerFuncs {
            add_func: Some(g.add_resource_slice),
            update_func: None,
            delete_func: Some(g.delete_resource_slice),
        });
        synced.push(slice_handler.has_synced);
    }

    if let Some(pcrs) = pcrs {
        let pcr_handler = pcrs.informer().add_event_handler(ResourceEventHandlerFuncs {
            add_func: Some(g.add_pcr),
            update_func: None,
            delete_func: Some(g.delete_pcr),
        });
        synced.push(pcr_handler.has_synced);
    }

    std::thread::spawn(move || {
        wait_for_named_cache_sync("node_authorizer", NeverStop, synced);
    });
}
