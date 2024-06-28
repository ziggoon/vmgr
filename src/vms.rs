use core::slice;
use std::ffi::CStr;
use std::time::Instant;

use chrono::prelude::*;

use virt::connect::Connect;
use virt::domain::{Domain, DomainStatsRecord};
use virt::domain_snapshot::DomainSnapshot;
use virt::error::Error;
use virt::sys::{
    virDomainGetID, virDomainGetName, virDomainStatsRecord,
    VIR_CONNECT_GET_ALL_DOMAINS_STATS_ACTIVE, VIR_CONNECT_GET_ALL_DOMAINS_STATS_INACTIVE,
    VIR_DOMAIN_SNAPSHOT_CREATE_DISK_ONLY, VIR_DOMAIN_STATS_BALLOON, VIR_DOMAIN_STATS_BLOCK,
    VIR_DOMAIN_STATS_CPU_TOTAL, VIR_DOMAIN_STATS_INTERFACE, VIR_DOMAIN_STATS_STATE,
    VIR_DOMAIN_STATS_VCPU,
};

#[derive(Debug)]
pub struct VmMetrics {
    pub name: String,
    pub status: bool,
    pub id: u32,
    pub timestamp: Instant,
    pub cpu_time: u64,
    pub mem_rss: u64,
    pub mem_cache: u64,
    pub net_name: String,
    pub net_rx: u64,
    pub net_tx: u64,
    pub disk_name: String,
    pub disk_path: String,
    pub disk_rx: u64,
    pub disk_wx: u64,
}

impl Default for VmMetrics {
    fn default() -> Self {
        Self {
            name: String::from("unknown"),
            status: false,
            id: 0,
            timestamp: Instant::now(),
            cpu_time: 0,
            mem_rss: 0,
            mem_cache: 0,
            net_name: String::from("unknown"),
            net_rx: 0,
            net_tx: 0,
            disk_name: String::from("unknown"),
            disk_path: String::from("unknown"),
            disk_rx: 0,
            disk_wx: 0,
        }
    }
}

pub fn connect(uri: &str) -> Connect {
    let conn = match Connect::open(uri) {
        Ok(c) => c,
        Err(e) => panic!("failed to connect to hypervisor: {}", e),
    };

    return conn;
}

pub fn disconnect(conn: &mut Connect) {
    if let Err(e) = conn.close() {
        panic!("failed to disconnect from hypervisor: {}", e);
    }
}

pub fn get_vm_data(conn: &Connect) -> Vec<VmMetrics> {
    let domains = get_domain_stats(&conn).unwrap();
    let mut vm_data = vec![];

    for domain in domains {
        let mut vm_metrics = VmMetrics::default();
        vm_metrics.timestamp = Instant::now();

        let record_ptr: *const virDomainStatsRecord = domain.ptr;
        let domain_ptr = unsafe { (*record_ptr).dom };
        let params_ptr = unsafe { (*record_ptr).params };
        let nparams = unsafe { (*record_ptr).nparams };

        let name_ptr = unsafe { virDomainGetName(domain_ptr) };
        let domain_name = if !name_ptr.is_null() {
            unsafe { CStr::from_ptr(name_ptr).to_string_lossy().into_owned() }
        } else {
            String::from("unknown")
        };

        let id = unsafe { virDomainGetID(domain_ptr) };

        vm_metrics.name = domain_name;
        vm_metrics.id = id;

        let params = unsafe { slice::from_raw_parts(params_ptr, nparams as usize) };
        for param in params {
            let field = unsafe { CStr::from_ptr(param.field.as_ptr()) }.to_string_lossy();

            match field.as_ref() {
                "state.state" => {
                    vm_metrics.status = if unsafe { param.value.ul } == 1 {
                        true
                    } else {
                        false
                    }
                }
                "cpu.time" => vm_metrics.cpu_time = unsafe { param.value.ul },
                "balloon.rss" => vm_metrics.mem_rss = unsafe { param.value.ul },
                "balloon.disk_caches" => vm_metrics.mem_cache = unsafe { param.value.ul },
                "net.0.name" => {
                    vm_metrics.net_name =
                        unsafe { CStr::from_ptr(param.value.s).to_string_lossy().to_string() }
                }
                "net.0.rx.bytes" => vm_metrics.net_rx = unsafe { param.value.ul },
                "net.0.tx.bytes" => vm_metrics.net_tx = unsafe { param.value.ul },
                "block.0.name" => {
                    vm_metrics.disk_name =
                        unsafe { CStr::from_ptr(param.value.s).to_string_lossy().to_string() }
                }
                "block.0.path" => {
                    vm_metrics.disk_path =
                        unsafe { CStr::from_ptr(param.value.s).to_string_lossy().to_string() }
                }
                "block.0.rd.bytes" => vm_metrics.disk_rx = unsafe { param.value.ul },
                "block.0.wd.bytes" => vm_metrics.disk_wx = unsafe { param.value.ul },

                _ => {}
            }
        }
        vm_data.push(vm_metrics);
    }
    return vm_data;
}

fn get_domain_stats(conn: &Connect) -> Result<Vec<DomainStatsRecord>, Error> {
    let stats_flags = VIR_DOMAIN_STATS_STATE
        | VIR_DOMAIN_STATS_CPU_TOTAL
        | VIR_DOMAIN_STATS_BALLOON
        | VIR_DOMAIN_STATS_VCPU
        | VIR_DOMAIN_STATS_INTERFACE
        | VIR_DOMAIN_STATS_BLOCK;

    conn.get_all_domain_stats(
        stats_flags,
        VIR_CONNECT_GET_ALL_DOMAINS_STATS_ACTIVE | VIR_CONNECT_GET_ALL_DOMAINS_STATS_INACTIVE,
    )
}

pub fn snapshot(conn: &Connect, name: &str) {
    if let Ok(dom) = Domain::lookup_by_name(&conn, &name) {
        let xml = format!(
            r#"
                <domainsnapshot>
                    <name>{}-{}</name>
                    <description>vmgr snapshot</description>
                </domainsnapshot>
            "#,
            name,
            Utc::now().to_string()
        );

        let mut snapshot =
            DomainSnapshot::create_xml(&dom, &xml, VIR_DOMAIN_SNAPSHOT_CREATE_DISK_ONLY).unwrap();
        snapshot.free().unwrap();
    }
}

pub fn start(conn: &Connect, name: &str) {
    if let Ok(dom) = Domain::lookup_by_name(&conn, &name) {
        dom.create().unwrap();
    }
}

pub fn stop(conn: &Connect, name: &str) {
    if let Ok(dom) = Domain::lookup_by_name(&conn, &name) {
        dom.destroy().unwrap();
    }
}
