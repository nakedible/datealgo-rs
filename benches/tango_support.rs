//! Support for the tango benchmarks, which `scripts/ab_lib.sh` runs.

/// Pin a tango worker process to the CPU given for its role in
/// `AB_CPU_CANDIDATE` or `AB_CPU_BASELINE`. Tango runs the two builds
/// at the same time on different CPUs, and CPUs can differ in speed, so
/// `scripts/ab_lib.sh` chooses which build runs on which CPU, and swaps them.
pub fn pin_worker() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) != Some("__worker") {
        return;
    }
    let Some(role) = args.windows(2).find(|w| w[0] == "--role").map(|w| w[1].to_uppercase()) else {
        return;
    };
    let Ok(cpu) = std::env::var(format!("AB_CPU_{role}")) else {
        return;
    };
    let cpu: usize = cpu.parse().expect("CPU number");
    #[cfg(target_os = "linux")]
    // SAFETY: the set is initialized before it is passed to the kernel
    unsafe {
        let mut set: libc::cpu_set_t = std::mem::zeroed();
        libc::CPU_SET(cpu, &mut set);
        let ret = libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &set);
        assert_eq!(ret, 0, "could not pin to CPU {cpu}");
    }
}
